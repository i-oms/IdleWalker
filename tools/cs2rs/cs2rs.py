#!/usr/bin/env python3
"""One-shot translator used for the C# -> Rust migration of IdleWalker.

It handles exactly the C# 5 subset used by the Domain and Rendering layers:
it parses them, infers expression types (C# converts int to double implicitly,
Rust does not) and emits Rust that keeps the original statement order, so the
floating point results stay identical.

Object model of the output: a C# class becomes a Copy handle `Foo` around a
heap `FooData`; methods take the handle by value, so walkers can call each
other re-entrantly as they do in C#, with no long-lived Rust references.
See crates/domain/src/rt.rs.

Usage: cs2rs.py <csharp src dir> <rust src dir>
"""
import re, sys, os, json

# ------------------------------------------------------------------ lexer
TOK = re.compile(r'''
 (?P<ws>\s+)|(?P<lc>//[^\n]*)|(?P<bc>/\*.*?\*/)|(?P<attr_skip>^\s*\#[^\n]*)|
 (?P<num>0[xX][0-9a-fA-F]+[uUlL]*|(?:\d+\.\d*|\.\d+|\d+)(?:[eE][+-]?\d+)?[dDfFuUlLmM]*)|
 (?P<id>@?[A-Za-z_][A-Za-z_0-9]*)|
 (?P<str>@"(?:[^"]|"")*"|"(?:\\.|[^"\\])*")|(?P<chr>'(?:\\.|[^'\\])')|
 (?P<op><<=|>>=|\+\+|--|\+=|-=|\*=|/=|%=|&=|\|=|\^=|==|!=|<=|>=|&&|\|\||\?\?|<<|[{}()\[\];,.:?+\-*/%&|^~!<>=])
''', re.X | re.S | re.M)

class T:
    __slots__ = ('k', 'v', 'lead', 'trail', 'line')
    def __init__(s, k, v, line): s.k, s.v, s.line = k, v, line; s.lead = []; s.trail = []
    def __repr__(s): return f'{s.v!r}@{s.line}'

def lex(src):
    out, pos, line, pending = [], 0, 1, []
    while pos < len(src):
        m = TOK.match(src, pos)
        if not m: raise SyntaxError(f'lex at line {line}: {src[pos:pos+30]!r}')
        kind = m.lastgroup; text = m.group(kind)
        if kind in ('lc', 'bc'):
            if out and out[-1].line == line and not pending: out[-1].trail.append(text)
            else: pending.append(text)
        elif kind not in ('ws', 'attr_skip'):
            t = T(kind, text, line); t.lead = pending; pending = []; out.append(t)
        line += text.count('\n'); pos = m.end()
    out.append(T('eof', '<eof>', line)); out[-1].lead = pending
    return out

# ------------------------------------------------------------------ AST: tuples/dicts
class N(dict):
    # Node fields win over dict methods (nodes have .items, .get, .set fields).
    def __getattribute__(self, k):
        if dict.__contains__(self, k): return dict.__getitem__(self, k)
        if k.startswith('__') or k in ('keys', 'values'): return object.__getattribute__(self, k)
        if k == 'get': return lambda key, d=None: dict.get(self, key, d)
        return None

PRIMS = {'int','double','bool','string','byte','uint','long','ulong','char','float','void','object','short'}
MODS = {'public','private','internal','protected','static','readonly','const','sealed','abstract','partial','virtual','override','unsafe','new','extern','volatile'}

class Parser:
    def __init__(s, toks): s.t, s.i = toks, 0
    @property
    def p(s): return s.t[s.i]
    def peek(s, n=1): return s.t[min(s.i+n, len(s.t)-1)]
    def at(s, v): return s.p.v == v and s.p.k in ('op', 'id')
    def eat(s, v=None):
        t = s.p
        if v is not None and t.v != v: raise SyntaxError(f'expected {v!r} got {t}')
        s.i += 1; return t
    def opt(s, v):
        if s.at(v): s.i += 1; return True
        return False

    # types --------------------------------------------------------------
    def try_type(s):
        save = s.i
        try:
            ty = s.type_()
            return ty
        except SyntaxError:
            s.i = save; return None
    def type_(s):
        t = s.p
        if t.k != 'id': raise SyntaxError('type')
        name = t.v; s.i += 1
        while s.at('.') and s.peek().k == 'id':
            s.i += 1; name = s.eat().v     # qualified names: keep the last part
        if s.at('<'):
            s.i += 1; args = [s.type_()]
            while s.opt(','): args.append(s.type_())
            if s.at('>'): s.i += 1
            else: raise SyntaxError('generic')
            name = f"{name}<{','.join(args)}>"
        if s.at('?') and name not in ():      # nullable: only when a declarator-ish thing follows
            nx = s.peek()
            if (nx.k == 'id' and s.peek(2).v in ('=', ';', ',', ')', '(', '{')) or nx.v in ('>', ')', ','):
                s.i += 1; name += '?'
        while s.at('[') and s.peek().v in (']', ','):
            s.i += 1; dims = 1
            while s.opt(','): dims += 1
            s.eat(']'); name += '[]' if dims == 1 else '[,]'
        return name

    # top level ----------------------------------------------------------
    def file(s):
        items = []
        while s.p.k != 'eof':
            if s.at('using'):
                while not s.opt(';'): s.i += 1
            elif s.at('namespace'):
                s.i += 1
                while not s.at('{'): s.i += 1
                s.eat('{')
                while not s.at('}'):
                    if s.at('using'):
                        while not s.opt(';'): s.i += 1
                    else: items.append(s.type_decl())
                s.eat('}')
            else: items.append(s.type_decl())
        return items
    def attrs(s):
        found = []
        while s.at('['):
            d = 0
            while True:
                if s.at('['): d += 1
                if s.at(']'): d -= 1
                if s.p.k == 'id': found.append(s.p.v)
                s.i += 1
                if d == 0: break
        return found
    def mods(s):
        m = set()
        while s.p.v in MODS and s.p.k == 'id': m.add(s.eat().v)
        return m
    def type_decl(s):
        lead = s.p.lead; s.attrs(); m = s.mods()
        kind = s.eat().v
        name = s.eat().v
        if kind == 'enum':
            s.eat('{'); vals = []
            while not s.at('}'):
                t = s.eat(); c = list(t.lead); v = None
                if s.opt('='): v = s.expr()
                vals.append((t.v, v, c, s.t[s.i-1].trail))
                if s.at(','):
                    tk = s.eat(); vals[-1] = (t.v, v, c, s.t[s.i-2].trail + tk.trail)
            s.eat('}'); s.opt(';')
            return N(node='enum', name=name, vals=vals, lead=lead)
        bases = []
        if s.opt(':'):
            bases.append(s.type_())
            while s.opt(','): bases.append(s.type_())
        s.eat('{'); members = []; nested = []
        while not s.at('}'):
            r = s.member(name, nested)
            if r: members.extend(r)
        s.eat('}'); s.opt(';')
        return N(node=kind, name=name, mods=m, bases=bases, members=members, nested=nested, lead=lead)
    def member(s, cname, nested):
        lead = s.p.lead; save = s.i
        attrs = s.attrs(); m = s.mods()
        if s.p.v in ('class', 'struct', 'enum', 'interface'):
            s.i = save; nested.append(s.type_decl()); return None
        # constructor
        if s.p.v == cname and s.peek().v == '(':
            s.i += 1; params = s.params(); init = None
            if s.opt(':'):
                k = s.eat().v; args = s.args(); init = (k, args)
            body = s.block() if s.at('{') else None
            if body is None: s.eat(';')
            return [N(node='ctor', params=params, init=init, body=body, mods=m, lead=lead)]
        ty = s.type_()
        name = s.eat().v
        while s.at('.'): s.i += 1; name = s.eat().v
        if s.at('('):
            params = s.params(); body = None
            if s.at('{'): body = s.block()
            else: s.eat(';')
            return [N(node='method', type=ty, name=name, params=params, body=body, mods=m, lead=lead)]
        if s.at('{'):
            s.eat('{'); get = set_ = None; auto = True
            while not s.at('}'):
                s.mods(); w = s.eat().v
                if s.at('{'):
                    b = s.block(); auto = False
                    if w == 'get': get = b
                    else: set_ = b
                else:
                    s.eat(';')
                    if w == 'get': get = True
                    else: set_ = True
            s.eat('}')
            return [N(node='prop', type=ty, name=name, get=get, set=set_, auto=auto, mods=m, lead=lead)]
        out = []
        while True:
            init = s.expr_or_array() if s.opt('=') else None
            out.append(N(node='field', type=ty, name=name, init=init, mods=m, attrs=attrs, lead=lead, trail=[]))
            lead = []
            if s.opt(','):
                t = s.eat(); name = t.v
                continue
            break
        out[-1].trail = s.p.trail; s.eat(';')
        return out
    def params(s):
        s.eat('('); ps = []
        while not s.at(')'):
            mod = None
            if s.p.v in ('ref', 'out', 'params') and s.p.k == 'id': mod = s.eat().v
            ty = s.type_(); name = s.eat().v
            dflt = s.expr() if s.opt('=') else None
            ps.append(N(type=ty, name=name, mod=mod, dflt=dflt))
            if not s.opt(','): break
        s.eat(')'); return ps
    def args(s):
        s.eat('('); a = []
        while not s.at(')'):
            mod = None
            named = None
            if s.p.k == 'id' and s.peek().v == ':': named = s.eat().v; s.i += 1
            if s.p.v in ('ref', 'out') and s.p.k == 'id': mod = s.eat().v
            e = s.expr()
            if named: e = N(e='named', x=e, name=named)
            a.append(N(e='refarg', x=e, mod=mod) if mod else e)
            if not s.opt(','): break
        s.eat(')'); return a

    # statements ---------------------------------------------------------
    def block(s):
        s.eat('{'); st = []
        while not s.at('}'): st.append(s.stmt())
        end = s.eat('}')
        return N(s='block', body=st, endlead=end.lead)
    def looks_decl(s):
        save = s.i
        try:
            if s.p.k != 'id' or s.p.v in ('return','new','this','base','throw','null','true','false','typeof','default'): return False
            s.type_()
            return s.p.k == 'id' and s.peek().v in ('=', ';', ',')
        except SyntaxError: return False
        finally: s.i = save
    def stmt(s):
        lead = s.p.lead; st = s.stmt_(); st['lead'] = lead
        st['trail'] = s.t[s.i-1].trail
        return st
    def stmt_(s):
        t = s.p
        if t.v == '{': return s.block()
        if t.v == ';': s.i += 1; return N(s='empty')
        if t.k == 'id':
            v = t.v
            if v == 'if':
                s.i += 1; s.eat('('); c = s.expr(); s.eat(')')
                a = s.stmt(); b = s.stmt() if s.opt('else') else None
                return N(s='if', c=c, a=a, b=b)
            if v == 'while':
                s.i += 1; s.eat('('); c = s.expr(); s.eat(')')
                return N(s='while', c=c, body=s.stmt())
            if v == 'do':
                s.i += 1; b = s.stmt(); s.eat('while'); s.eat('('); c = s.expr(); s.eat(')'); s.eat(';')
                return N(s='do', c=c, body=b)
            if v == 'for':
                s.i += 1; s.eat('('); init = []
                if not s.at(';'):
                    if s.looks_decl(): init = [s.decl()]
                    else:
                        init = [N(s='expr', x=s.expr())]
                        while s.opt(','): init.append(N(s='expr', x=s.expr()))
                s.eat(';'); c = None if s.at(';') else s.expr(); s.eat(';'); it = []
                while not s.at(')'):
                    it.append(s.expr())
                    if not s.opt(','): break
                s.eat(')')
                return N(s='for', init=init, c=c, it=it, body=s.stmt())
            if v == 'foreach':
                s.i += 1; s.eat('('); ty = s.type_(); name = s.eat().v; s.eat('in'); c = s.expr(); s.eat(')')
                return N(s='foreach', type=ty, name=name, coll=c, body=s.stmt())
            if v == 'return':
                s.i += 1; x = None if s.at(';') else s.expr(); s.eat(';')
                return N(s='return', x=x)
            if v in ('break', 'continue'):
                s.i += 1; s.eat(';'); return N(s=v)
            if v == 'throw':
                s.i += 1
                if not s.at(';'): s.expr()
                s.eat(';'); return N(s='throw')
            if v == 'switch':
                s.i += 1; s.eat('('); x = s.expr(); s.eat(')'); s.eat('{'); cases = []
                while not s.at('}'):
                    labels = []
                    while s.at('case') or s.at('default'):
                        if s.opt('default'): labels.append(None)
                        else: s.i += 1; labels.append(s.expr())
                        s.eat(':')
                    body = []
                    while not (s.at('case') or s.at('default') or s.at('}')): body.append(s.stmt())
                    cases.append((labels, body))
                s.eat('}')
                return N(s='switch', x=x, cases=cases)
            if v == 'try':
                s.i += 1; b = s.block(); c = None; f = None
                while s.at('catch'):
                    s.i += 1
                    if s.at('('):
                        while not s.opt(')'): s.i += 1
                    c = s.block()
                if s.opt('finally'): f = s.block()
                return N(s='try', body=b, catch=c, fin=f)
            if v == 'const':
                s.i += 1; d = s.decl(); s.eat(';'); return d
            if s.looks_decl():
                d = s.decl(); s.eat(';'); return d
        x = s.expr(); s.eat(';')
        return N(s='expr', x=x)
    def decl(s):
        ty = s.type_(); vs = []
        while True:
            name = s.eat().v; init = s.expr_or_array() if s.opt('=') else None
            vs.append((name, init))
            if not s.opt(','): break
        return N(s='decl', type=ty, vars=vs)

    # expressions --------------------------------------------------------
    def expr_or_array(s):
        if s.at('{'): return s.array_init()
        return s.expr()
    def array_init(s):
        s.eat('{'); items = []
        while not s.at('}'):
            items.append(s.expr_or_array())
            if not s.opt(','): break
        s.eat('}'); return N(e='arrayinit', items=items)
    def expr(s):
        lhs = s.cond()
        if s.p.k == 'op' and s.p.v in ('=', '+=', '-=', '*=', '/=', '%=', '&=', '|=', '^=', '<<=', '>>='):
            op = s.eat().v; rhs = s.expr()
            return N(e='assign', op=op, l=lhs, r=rhs)
        if s.at('>') and s.peek().v == '>' and s.peek(2).v == '=':
            s.i += 3; return N(e='assign', op='>>=', l=lhs, r=s.expr())
        return lhs
    def cond(s):
        c = s.bin(0)
        if s.at('?'):
            s.i += 1; a = s.expr(); s.eat(':'); b = s.expr()
            return N(e='cond', c=c, a=a, b=b)
        return c
    LEVELS = [['??'], ['||'], ['&&'], ['|'], ['^'], ['&'], ['==', '!='], ['<', '>', '<=', '>='], ['<<', '>>'], ['+', '-'], ['*', '/', '%']]
    def bin(s, lv):
        if lv == len(s.LEVELS): return s.unary()
        l = s.bin(lv+1)
        while True:
            ops = s.LEVELS[lv]
            if '>>' in ops and s.at('>') and s.peek().v == '>' and s.peek(2).v != '=':
                s.i += 2; l = N(e='bin', op='>>', l=l, r=s.bin(lv+1)); continue
            if s.p.k == 'op' and s.p.v in ops:
                if s.p.v == '>' and s.peek().v == '>': break
                op = s.eat().v; l = N(e='bin', op=op, l=l, r=s.bin(lv+1))
            else: break
        return l
    def unary(s):
        t = s.p
        if t.k == 'op' and t.v in ('-', '+', '!', '~'):
            s.i += 1; return N(e='un', op=t.v, x=s.unary())
        if t.k == 'op' and t.v in ('++', '--'):
            s.i += 1; return N(e='pre', op=t.v, x=s.unary())
        if t.v == '(':
            save = s.i; s.i += 1; ty = s.try_type()
            if ty and s.at(')'):
                nx = s.peek()
                prim = ty.rstrip('[],?') in PRIMS
                if prim or nx.k in ('id', 'num', 'str', 'chr') or nx.v in ('(', '!', '~'):
                    if nx.k != 'op' or nx.v in ('(', '!', '~') or (prim and nx.v in ('-', '+')):
                        s.i += 1; return N(e='cast', type=ty, x=s.unary())
            s.i = save
        return s.postfix(s.primary())
    def primary(s):
        t = s.eat()
        if t.k == 'num': return N(e='num', v=t.v)
        if t.k == 'str': return N(e='str', v=t.v)
        if t.k == 'chr': return N(e='chr', v=t.v)
        if t.v == '(':
            x = s.expr(); s.eat(')'); return N(e='paren', x=x)
        if t.v == 'true' or t.v == 'false': return N(e='bool', v=t.v)
        if t.v == 'null': return N(e='null')
        if t.v == 'this': return N(e='this')
        if t.v == 'base': return N(e='this')
        if t.v in ('checked', 'unchecked'):
            s.eat('('); x = s.expr(); s.eat(')'); return N(e='paren', x=x)
        if t.v == 'default':
            s.eat('('); ty = s.type_(); s.eat(')'); return N(e='default', type=ty)
        if t.v == 'new':
            if s.at('['):
                s.eat('['); s.eat(']'); return N(e='arrayinit', items=s.array_init().items, type=None)
            base = s.p.v; s.i += 1
            while s.at('.'): s.i += 1; base = s.eat().v
            if s.at('<'):
                s.i += 1; a = [s.type_()]
                while s.opt(','): a.append(s.type_())
                s.eat('>'); base = f"{base}<{','.join(a)}>"
            if s.at('['):
                s.eat('[')
                if s.at(']') or s.at(','):
                    dims = 1
                    while s.opt(','): dims += 1
                    s.eat(']')
                    ty = base + ('[]' if dims == 1 else '[,]')
                    return N(e='arrayinit', items=s.array_init().items, type=ty)
                sizes = [s.expr()]
                while s.opt(','): sizes.append(s.expr())
                s.eat(']')
                while s.at('[') and s.peek().v == ']':
                    s.i += 2; base += '[]'
                return N(e='newarray', base=base, sizes=sizes)
            args = s.args()
            return N(e='new', type=base, args=args)
        if t.k == 'id':
            if t.v in PRIMS: return N(e='typename', name=t.v)
            return N(e='name', name=t.v)
        raise SyntaxError(f'unexpected {t}')
    def postfix(s, x):
        while True:
            if s.at('.'):
                s.i += 1; x = N(e='member', x=x, name=s.eat().v)
            elif s.at('('):
                x = N(e='call', f=x, args=s.args())
            elif s.at('['):
                s.i += 1; idx = [s.expr()]
                while s.opt(','): idx.append(s.expr())
                s.eat(']'); x = N(e='index', x=x, idx=idx)
            elif s.at('++') or s.at('--'):
                x = N(e='post', op=s.eat().v, x=x)
            else: return x

# ------------------------------------------------------------------ names
KEYWORDS = {'as','box','break','const','continue','crate','dyn','else','enum','extern','false','fn','for','if','impl','in','let','loop','match','mod','move','mut','pub','ref','return','self','static','struct','super','trait','true','type','unsafe','use','where','while','async','await','try','final','abstract','override','virtual','yield','do','macro','priv','typeof','unsized','become'}
def snake(n):
    n = n.lstrip('@')
    s = re.sub(r'(?<=[a-z0-9])([A-Z])', r'_\1', n)
    s = re.sub(r'([A-Z])([A-Z][a-z])', r'\1_\2', s).lower()
    return 'r#' + s if s in KEYWORDS else s
def shout(n):
    return snake(n).replace('r#', '').upper()

# ------------------------------------------------------------------ type helpers
NUM_RANK = {'byte':1,'short':2,'char':2,'int':3,'uint':4,'long':5,'ulong':6,'float':7,'double':8}
RS = {'int':'i32','double':'f64','bool':'bool','string':'String','byte':'u8','uint':'u32','long':'i64','ulong':'u64','char':'char','float':'f32','void':'()','short':'i16'}
def split_args(s):
    out, d, cur = [], 0, ''
    for ch in s:
        if ch == '<': d += 1
        if ch == '>': d -= 1
        if ch == ',' and d == 0: out.append(cur); cur = ''
        else: cur += ch
    out.append(cur); return out
def norm(t):
    if t.startswith('IList<'): return 'List<' + t[6:]
    return t
def rs(t):
    t = norm(t)
    if t.endswith('[,]'): return f'Arr2<{rs(t[:-3])}>'
    if t.endswith('[]'): return f'Arr<{rs(t[:-2])}>'
    if t.endswith('?'): return f'Option<{rs(t[:-1])}>'
    m = re.match(r'(\w+)<(.*)>$', t)
    if m:
        n, a = m.group(1), [rs(x) for x in split_args(m.group(2))]
        n = {'Dictionary':'Dict','KeyValuePair':'Kv','List':'Arr','Queue':'Arr','Stack':'Arr'}.get(n, n)
        return f"{n}<{','.join(a)}>"
    return RS.get(t, t)
def is_num(t): return t in NUM_RANK
def elem(t):
    t = norm(t)
    if t.endswith('[,]'): return t[:-3]
    if t.endswith('[]'): return t[:-2]
    m = re.match(r'(List|Queue|Stack)<(.*)>$', t)
    if m: return m.group(2)
    m = re.match(r'Dictionary<(.*)>$', t)
    if m:
        a = split_args(m.group(1)); return f'KeyValuePair<{a[0]},{a[1]}>'
    if t == 'string': return 'char'
    raise KeyError('elem ' + t)

class G:
    pass

# Library surface the translated code uses; the Rust side lives in rt.rs.
PRELUDE = r'''
struct Point { public int X, Y; public Point(int x, int y); public static Point Empty; }
struct Rectangle { public int X, Y, Width, Height; public Rectangle(int x, int y, int w, int h);
  public int Left {get;} public int Right {get;} public int Top {get;} public int Bottom {get;} public bool IsEmpty {get;}
  public static Rectangle FromLTRB(int l, int t, int r, int b); public bool Contains(Point p); public bool Contains(Rectangle r); public bool Contains(int x, int y); public static Rectangle Empty; }
class Random { public Random(); public Random(int seed); public int Next(int n); public int Next(int a, int b); public double NextDouble(); }
class StringBuilder { public StringBuilder(); public StringBuilder Append(object x); public int Length {get;set;} public string ToString(); }
interface IWorld { int Left {get;} int Right {get;} int[] MonitorEdges {get;} Rectangle MonitorBoundsAt(Point p); double GroundAt(double x); int ScaleAt(double x); }
'''

class Cls:
    def __init__(s, name, kind): s.name, s.kind, s.fields, s.props, s.methods, s.ctors, s.consts, s.decl = name, kind, {}, {}, {}, [], {}, None; s.statics = {}

class Tr:
    def __init__(s):
        s.classes = {}; s.enums = {}; s.outputs = {}; s.statics_out = []; s.label = 0

    # ---- collection -------------------------------------------------------
    def add_type(s, d, fname, extern=False):
        if d.node == 'enum':
            s.enums[d.name] = d; d['file'] = fname; d['extern'] = extern; return
        c = s.classes.get(d.name)
        if c is None:
            c = Cls(d.name, d.node); c.decls = []; c.file = fname; c.extern = extern; c.base = None
            s.classes[d.name] = c
        for b in d.bases:
            if b != 'IDisposable' and not b.startswith('I') : c.base = b
        c.decls.append((d, fname))
        for n in d.nested: s.add_type(n, fname, extern)
        for m in d.members: s.add_member(c, m)
    def add_member(s, c, m):
        if m.node == 'field':
            if 'const' in m.mods: c.consts[m.name] = m
            elif 'static' in m.mods: c.statics[m.name] = m
            else: c.fields[m.name] = m
        elif m.node == 'prop': c.props[m.name] = m
        elif m.node == 'method': c.methods.setdefault(m.name, []).append(m)
        elif m.node == 'ctor': c.ctors.append(m)

    def flatten(s):
        for c in s.classes.values():
            if c.base and c.base in s.classes:
                b = s.classes[c.base]; c.basecls = b
                for k in ('fields', 'props', 'consts', 'statics'):
                    for n, m in getattr(b, k).items():
                        getattr(c, k).setdefault(n, m)
                for n, ms in b.methods.items():
                    if n in c.methods and all('abstract' in x.mods for x in ms): continue
                    if n in c.methods:
                        c.methods[n] = c.methods[n] + [x for x in ms if 'abstract' not in x.mods and not any(len(x.params)==len(y.params) for y in c.methods[n])]
                    else: c.methods[n] = [x for x in ms if 'abstract' not in x.mods]
                c.methods = {k:v for k,v in c.methods.items() if v}

    def is_copy(s, t):
        t = norm(t)
        if t in RS: return t != 'string'
        if t.endswith('?'): return s.is_copy(t[:-1])
        if t in s.enums: return True
        if t.endswith(']') or '<' in t: return False
        c = s.classes.get(t)
        if c is None: raise KeyError('type ' + t)
        if c.kind == 'struct':
            return all(s.is_copy(f.type) for f in c.fields.values())
        return True

    def assign_names(s):
        for c in s.classes.values():
            used = {}
            c.mname = {}; c.fname = {}
            seen = set()
            for n in c.fields:
                r = snake(n)
                while r in seen: r += '_'
                seen.add(r); c.fname[n] = r
            for n in c.props:
                if c.props[n].auto and c.kind != 'interface':
                    r = snake(n) + '_v'
                    c.fname[n] = r
            names = set()
            for n in c.props:
                names.add(snake(n)); c.mname['P:' + n] = snake(n); c.mname['S:' + n] = 'set_' + snake(n).replace('r#', '')
            for n, ms in c.methods.items():
                for i, m in enumerate(ms):
                    r = snake(n)
                    if len(ms) > 1 and i > 0: r = f"{r.replace('r#','')}_{i+1}"
                    while r in names:
                        r = r.replace('r#','') + '_'
                    if len(ms) == 1 or i == len(ms) - 1: names.add(r)
                    m['rname'] = r
            for i, m in enumerate(c.ctors):
                m['rname'] = 'new' if i == 0 else f'new_{i+1}'
                m['iname'] = 'ctor' if i == 0 else f'ctor_{i+1}'

    # ---- member resolution ---------------------------------------------------
    def static_ident(s, cname, n):
        return f'{cname.upper()}_{shout(n)}'

    def choose(s, overloads, argtypes):
        def fits(m):
            n = len(argtypes); ps = m.params
            if n == len(ps): return True
            if ps and ps[-1].mod == 'params' and n >= len(ps) - 1: return True
            return n < len(ps) and all(p.dflt is not None for p in ps[n:])
        cands = [m for m in overloads if fits(m)]
        if not cands: raise KeyError(f'no overload for {[m.get("name") for m in overloads]} args {argtypes}')
        if len(cands) == 1: return cands[0]
        best, bs = None, -1
        for m in cands:
            score = 0
            for p, a in zip(m.params, argtypes):
                pt = norm(p.type)
                if pt == a: score += 3
                elif is_num(pt) and is_num(a) and NUM_RANK[a] <= NUM_RANK[pt]: score += 1
                elif a == 'null': score += 1
                elif pt == 'object': score += 0
                else: score -= 10
            if len(m.params) == len(argtypes): score += 1
            if score > bs: best, bs = m, score
        return best


class R:
    """Expression result: Rust code, C# type, is it a place (needs .clone() to move)."""
    __slots__ = ('c', 't', 'place')
    def __init__(s, c, t, place=False): s.c, s.t, s.place = c, t, place


class Fn:
    """Translates one method body."""
    def __init__(s, tr, cls, is_static, ret=None):
        s.tr, s.cls, s.is_static, s.ret = tr, cls, is_static, ret
        s.scopes = [{}]; s.brk = []; s.refparams = set(); s.label = 0

    def push(s): s.scopes.append({})
    def pop(s): s.scopes.pop()
    def local(s, n, t): s.scopes[-1][n] = t
    def lookup(s, n):
        for sc in reversed(s.scopes):
            if n in sc: return sc[n]
        return None
    @property
    def selfc(s): return 'self'

    # ---- conversions --------------------------------------------------------
    def conv(s, r, to):
        """Implicit C# conversion of r to type `to`, then move out of places."""
        to = norm(to)
        if r.t == 'null':
            return s.null_of(to)
        if r.t != to and is_num(r.t) and is_num(to):
            c = r.c
            if re.fullmatch(r'-?\d+', c) and to in ('double', 'float'): c = c + '.0'
            elif re.fullmatch(r'-?\d+', c): pass
            elif re.fullmatch(r"'.*'", c): c = f'({c} as u32 as {RS[to]})'
            elif r.t == 'char': c = f'({c} as u32 as {RS[to]})'
            else: c = f'(({c}) as {RS[to]})'
            return c
        if to.endswith('?') and not r.t.endswith('?'):
            return f'Some({s.conv(r, to[:-1])})'
        return s.mv(r)
    def mv(s, r):
        if r.place and not s.tr.is_copy(r.t): return r.c + '.clone()'
        return r.c
    def null_of(s, to):
        if to.endswith('?'): return 'None'
        if to == 'string': return 'String::new()'
        return f'<{rs(to)}>::default()'
    def promote(s, a, b):
        if a == b: return a
        if not is_num(a) or not is_num(b): return a
        if 'double' in (a, b): return 'double'
        if 'float' in (a, b): return 'float'
        if 'ulong' in (a, b): return 'ulong'
        if 'long' in (a, b): return 'long'
        if 'uint' in (a, b):
            return 'long' if {a, b} & {'int', 'short'} else 'uint'
        return 'int'

    # ---- identifier / member access ------------------------------------------
    def field_on(s, base_code, cls, name, base_t):
        """Field, property or const `name` of class cls on an instance expression."""
        if name in cls.fields:
            return R(f'{base_code}.{cls.fname[name]}', norm(cls.fields[name].type), True)
        if name in cls.props:
            p = cls.props[name]
            return R(f'{base_code}.{cls.mname["P:" + name]}()', norm(p.type))
        return None
    def static_on(s, cls, name):
        if name in cls.consts:
            base = getattr(cls, 'basecls', None)
            own = cls if base is None or name not in base.consts else base
            return R(f'{own.name}::{shout(name)}', norm(cls.consts[name].type))
        if name in cls.statics:
            f = cls.statics[name]
            if cls.extern: return R(f'{cls.name}::default()', norm(f.type))
            owner = cls.name if name in s.own_statics(cls) else cls.basecls.name
            if 'ThreadStatic' in (f.attrs or []): return R('THREADSTATIC', norm(f.type), True)
            if 'readonly' not in f.mods: return R(f'(*{s.tr.static_ident(owner, name)}.at())', norm(f.type), True)
            return R(f'(*{s.tr.static_ident(owner, name)})', norm(f.type), True)
        if name in cls.props and 'static' in cls.props[name].mods:
            return R(f'{cls.name}::{cls.mname["P:" + name]}()', norm(cls.props[name].type))
        return None
    def own_statics(s, cls):
        own = set()
        for d, _ in cls.decls:
            for m in d.members:
                if m.node == 'field' and 'static' in m.mods: own.add(m.name)
        return own

    def name(s, n):
        t = s.lookup(n)
        if t is not None:
            c = snake(n)
            if n in s.refparams: return R(f'(*{c})', t, True)
            return R(c, t, True)
        c = s.cls
        while c:
            r = s.static_on(c, n)
            if r: return r
            if not s.is_static:
                p = c.props.get(n)
                if n in c.fields or (p and 'static' not in p.mods):
                    return s.field_on('self', c, n, c.name)
            c = getattr(c, 'outer', None)
        if n in s.tr.classes: return R(n, 'type:' + n)
        if n in s.tr.enums: return R(n, 'type:' + n)
        if n in ('TimeSpan', 'Math', 'Array', 'CultureInfo', 'NumberStyles', 'Stopwatch', 'DateTime', 'Environment', 'String'):
            return R(n, 'type:' + n)
        raise KeyError(f'unknown name {n} in {s.cls.name}')

    def color_color(s, x, n):
        """`Figure.Minion` where a member is also called Figure: C# means the type."""
        if x.e != 'name' or s.lookup(x.name) is not None: return None
        en = s.tr.enums.get(x.name)
        if en and any(v[0] == n for v in en.vals): return R(x.name, 'type:' + x.name)
        c = s.tr.classes.get(x.name)
        if c:
            if n in c.consts or n in c.statics: return R(x.name, 'type:' + x.name)
            if n in c.props and 'static' in c.props[n].mods: return R(x.name, 'type:' + x.name)
            if any('static' in m.mods for m in c.methods.get(n, [])): return R(x.name, 'type:' + x.name)
        return None

    def member(s, e, lhs=False):
        n = e.name
        if e.x.e == 'typename':
            tn = e.x.name
            if tn == 'double':
                return {'MaxValue': R('f64::MAX', 'double'), 'MinValue': R('f64::MIN', 'double'), 'NaN': R('f64::NAN', 'double'),
                        'PositiveInfinity': R('f64::INFINITY', 'double'), 'NegativeInfinity': R('f64::NEG_INFINITY', 'double')}[n]
            if tn == 'int':
                return {'MaxValue': R('i32::MAX', 'int'), 'MinValue': R('i32::MIN', 'int')}[n]
            if tn == 'string' and n == 'Empty': return R('String::new()', 'string')
            raise KeyError(f'{tn}.{n}')
        x = s.color_color(e.x, n) or s.expr(e.x, lhs=lhs)
        if x.t.startswith('type:'):
            tn = x.t[5:]
            if tn == 'Math':
                return {'PI': R('std::f64::consts::PI', 'double'), 'E': R('std::f64::consts::E', 'double')}[n]
            if tn in s.tr.enums: return R(f'{tn}::{n}', tn)
            if tn == 'TimeSpan': return R('36_000_000_000i64', 'long')
            if tn == 'CultureInfo': return R('()', 'culture')
            if tn == 'NumberStyles': return R('()', 'culture')
            if tn == 'Stopwatch':
                return {'Frequency': R('rt::stopwatch_frequency()', 'long')}[n]
            if tn == 'DateTime': return R({'Now': 'rt::DateTime::now()', 'UtcNow': 'rt::DateTime::utc_now()'}[n], 'DateTime')
            c = s.tr.classes[tn]
            r = s.static_on(c, n)
            if r: return r
            raise KeyError(f'{tn}.{n}')
        t = norm(x.t)
        if n in ('Length', 'Count') and (t.endswith(']') or t.startswith(('List<','Dictionary<','Queue<','Stack<'))):
            return R(f'{x.c}.len()', 'int')
        if n == 'Length' and t == 'string': return R(f'{x.c}.chars().count() as i32', 'int')
        if t.endswith('?'):
            if n == 'Value': return R(f'{x.c}.unwrap()', t[:-1])
            if n == 'HasValue': return R(f'{x.c}.is_some()', 'bool')
        if t.startswith('KeyValuePair<'):
            a = split_args(t[13:-1])
            return R(f'{x.c}.{ "key" if n == "Key" else "value"}', a[0] if n == 'Key' else a[1], True)
        if t.startswith('Dictionary<') and n == 'Keys':
            a = split_args(t[11:-1]); return R(f'{x.c}.keys()', f'{a[0]}[]')
        if t == 'DateTime':
            return R(f'{x.c}.{snake(n)}()', 'int' if n != 'Ticks' else 'long')
        c = s.tr.classes.get(t)
        if not c: raise KeyError(f'member {n} on {t}')
        r = s.field_on(x.c, c, n, t)
        if r: return r
        r = s.static_on(c, n)
        if r: return r
        raise KeyError(f'{t}.{n}')

    # ---- calls -----------------------------------------------------------------
    def args(s, m, args):
        """Arguments for method/ctor m, converted to parameter types."""
        out = []; ps = m.params
        if any(a.e == 'named' for a in args) or (len(args) < len(ps) and not (ps[-1].mod == 'params')):
            byname = {}; pos = []
            for a in args:
                if a.e == 'named': byname[a.name] = a.x
                else: pos.append(a)
            args = [pos[i] if i < len(pos) else byname.get(p.name, p.dflt) for i, p in enumerate(ps)]
        for i, a in enumerate(args):
            if ps and ps[-1].mod == 'params' and i >= len(ps) - 1:
                break
            p = ps[i]
            if a.e == 'refarg':
                r = s.expr(a.x, lhs=True)
                c = r.c
                if c.startswith('(*') and a.x.e == 'name' and a.x.name in s.refparams: out.append(f'&mut *{snake(a.x.name)}')
                else: out.append(f'rt::mref(&mut {c})')
            else:
                out.append(s.conv(s.expr(a, want=p.type), p.type))
        if ps and ps[-1].mod == 'params':
            k = len(ps) - 1; et = elem(ps[-1].type)
            rest = [s.conv(s.expr(a, want=et), et) for a in args[k:]]
            out.append(f'Arr::from_vec(vec![{", ".join(rest)}])')
        return out
    def argtypes(s, args):
        out = []
        for a in args:
            x = a.x if a.e in ('refarg', 'named') else a
            try:
                out.append(s.expr(x).t)
            except KeyError:
                raise
        return out

    def call(s, e):
        f = e.f
        if f.e == 'name':
            n = f.name
            c = s.cls
            while c:
                ms = c.methods.get(n)
                if ms:
                    m = s.tr.choose(ms, s.argtypes(e.args))
                    a = s.args(m, e.args)
                    if 'static' in m.mods: return R(f'{c.name}::{m.rname}({", ".join(a)})', norm(m.type))
                    return R(f'self.{m.rname}({", ".join(a)})', norm(m.type))
                c = getattr(c, 'outer', None)
            raise KeyError(f'method {n} in {s.cls.name}')
        if f.e != 'member': raise KeyError('call target')
        n = f.name
        if f.x.e == 'typename':
            return s.prim_static(f.x.name, n, e.args)
        x = s.color_color(f.x, n) or s.expr(f.x)
        if x.t.startswith('type:'):
            tn = x.t[5:]
            if tn in ('Math', 'Array', 'Stopwatch', 'String'):
                return s.lib_static(tn, n, e.args)
            c = s.tr.classes[tn]
            m = s.tr.choose(c.methods[n], s.argtypes(e.args))
            return R(f'{tn}::{m.rname}({", ".join(s.args(m, e.args))})', norm(m.type))
        return s.instance_call(x, n, e.args)

    def prim_static(s, tn, n, args):
        a = [s.expr(x.x if x.e == 'refarg' else x, lhs=x.e == 'refarg') for x in args]
        if n == 'Parse':
            return R(f'rt::parse_{RS[tn]}(&{a[0].c})', tn)
        if n == 'TryParse':
            out = a[-1].c
            return R(f'rt::try_parse_{RS[tn]}(&{a[0].c}, &mut {out})', 'bool')
        if tn == 'double':
            if n == 'IsNaN': return R(f'({s.conv(a[0],"double")}).is_nan()', 'bool')
            if n == 'IsInfinity': return R(f'({s.conv(a[0],"double")}).is_infinite()', 'bool')
        if tn == 'string' and n == 'Join':
            return R(f'rt::join({s.str_of(a[0])}, &{a[1].c})', 'string')
        if tn == 'string' and n == 'IsNullOrEmpty': return R(f'{a[0].c}.is_empty()', 'bool')
        raise KeyError(f'{tn}.{n}')
    def str_of(s, r):
        if r.t == 'char': return f'&{r.c}.to_string()'
        return f'&{r.c}'

    def lib_static(s, tn, n, args):
        if tn == 'Stopwatch': return R('rt::stopwatch_timestamp()', 'long')
        if tn == 'Array':
            if n == 'Resize':
                arr = s.expr(args[0].x, lhs=True); elemt = arr.t
                return R(f'rt::array_resize(&mut {arr.c}, {s.conv(s.expr(args[1]),"int")})', 'void')
            a = [s.expr(x) for x in args]
            if n == 'Clear': return R(f'{a[0].c}.clear_range({a[1].c}, {a[2].c})', 'void')
            if n == 'Copy':
                if len(a) == 3: return R(f'rt::array_copy(&{a[0].c}, 0, &{a[1].c}, 0, {a[2].c})', 'void')
                return R(f'rt::array_copy(&{a[0].c}, {a[1].c}, &{a[2].c}, {a[3].c}, {a[4].c})', 'void')
            raise KeyError('Array.' + n)
        a = [s.expr(x) for x in args]
        if len(a) >= 1:
            t = a[0].t
            for y in a[1:]: t = s.promote(t, y.t)
        cv = [s.conv(y, t) for y in a]
        fn = {'Sin':'sin','Cos':'cos','Sqrt':'sqrt','Tan':'tan','Atan':'atan','Asin':'asin','Acos':'acos','Exp':'exp','Log':'ln','Floor':'floor','Ceiling':'ceil','Atan2':None,'Pow':None}
        if n in ('Sin','Cos','Sqrt','Tan','Atan','Asin','Acos','Exp','Log','Floor','Ceiling'):
            return R(f'({s.conv(a[0], "double")}).{fn[n]}()', 'double')
        if n == 'Atan2': return R(f'({s.conv(a[0],"double")}).atan2({s.conv(a[1],"double")})', 'double')
        if n == 'Pow': return R(f'rt::pow({s.conv(a[0],"double")}, {s.conv(a[1],"double")})', 'double')
        if n == 'Abs':
            return R(f'({cv[0]}).abs()', t)
        if n in ('Min', 'Max'):
            return R(f'rt::{n.lower()}_{RS[t]}({cv[0]}, {cv[1]})', t)
        if n == 'Sign': return R(f'rt::sign_{RS[t]}({cv[0]})', 'int')
        if n == 'Round':
            if len(a) == 1: return R(f'rt::round_even({s.conv(a[0],"double")})', 'double')
            return R(f'rt::round_digits({s.conv(a[0],"double")}, {s.conv(a[1],"int")})', 'double')
        raise KeyError('Math.' + n)

    def instance_call(s, x, n, args):
        t = norm(x.t)
        def A(want=None):
            return [s.expr(y, want=want) for y in args]
        if t == 'string':
            a = A()
            if n == 'Split': return R(f'rt::split(&{x.c}, {a[0].c})', 'string[]')
            if n == 'Trim': return R(f'{x.c}.trim().to_string()', 'string')
            if n == 'Substring':
                if len(a) == 1: return R(f'rt::substr(&{x.c}, {a[0].c}, -1)', 'string')
                return R(f'rt::substr(&{x.c}, {a[0].c}, {a[1].c})', 'string')
            if n == 'IndexOf': return R(f'rt::index_of(&{x.c}, {a[0].c})', 'int')
            if n == 'StartsWith': return R(f'{x.c}.starts_with({a[0].c}.as_str())', 'bool')
            if n == 'ToString': return R(f'{x.c}.clone()', 'string')
        if n == 'ToString':
            if t in ('double', 'float', 'int', 'uint', 'long', 'ulong'):
                a = [y for y in A() if y.t == 'string']
                if a: return R(f'rt::format_num({s.conv(x,"double")}, &{a[0].c})', 'string')
                return R(f'rt::num_to_string({s.conv(x,"double")})' if t in ('double','float') else f'({x.c}).to_string()', 'string')
            return R(f'{x.c}.to_string()', 'string')
        if t.endswith('[]') or t.endswith('[,]') or t.startswith(('List<', 'Queue<', 'Stack<')):
            et = elem(t)
            if n == 'Clone': return R(f'{x.c}.deep_copy()', t)
            if n == 'GetLength': return R(f'{x.c}.get_length({s.conv(s.expr(args[0]),"int")})', 'int')
            if n == 'CopyTo':
                return R(f'rt::array_copy(&{x.c}, 0, &{s.expr(args[0]).c}, {s.conv(s.expr(args[1]),"int")}, {x.c}.len())', 'void')
            if n in ('Add', 'Push', 'Enqueue'): return R(f'{x.c}.add({s.conv(s.expr(args[0], want=et), et)})', 'void')
            if n == 'Clear': return R(f'{x.c}.clear()', 'void')
            if n == 'RemoveAt': return R(f'{x.c}.remove_at({s.conv(s.expr(args[0]),"int")})', 'void')
            if n == 'Sort': raise KeyError('Sort')
            if n == 'Contains': return R(f'{x.c}.contains({s.conv(s.expr(args[0], want=et), et)})', 'bool')
            raise KeyError(f'{t}.{n}')
        if t.startswith('Dictionary<'):
            k, v = split_args(t[11:-1])
            if n == 'TryGetValue':
                key = s.conv(s.expr(args[0]), k)
                if k == 'string' and args[0].e == 'str': pass
                o = s.expr(args[1].x, lhs=True)
                return R(f'{x.c}.try_get_value(&{key}, &mut {o.c})', 'bool')
            if n == 'ContainsKey': return R(f'{x.c}.contains_key(&{s.conv(s.expr(args[0]), k)})', 'bool')
            if n == 'Remove': return R(f'{x.c}.remove(&{s.conv(s.expr(args[0]), k)})', 'bool')
            if n == 'Clear': return R(f'{x.c}.clear()', 'void')
            if n == 'Add': return R(f'{x.c}.set({s.conv(s.expr(args[0]),k)}, {s.conv(s.expr(args[1]),v)})', 'void')
            raise KeyError(f'{t}.{n}')
        if t == 'StringBuilder':
            if n == 'Append':
                y = s.expr(args[0])
                if y.t == 'string': return R(f'{x.c}.append_str(&{y.c})', t)
                return R(f'{x.c}.append_disp({y.c})', t)
        if t in s.tr.enums and n == 'ToString': return R(f'{x.c}.to_string()', 'string')
        c = s.tr.classes.get(t)
        if not c: raise KeyError(f'call {n} on {t}')
        ms = c.methods.get(n)
        if not ms: raise KeyError(f'{t}.{n}()')
        m = s.tr.choose(ms, s.argtypes(args))
        if 'static' in m.mods: return R(f'{c.name}::{m.rname}({", ".join(s.args(m, args))})', norm(m.type))
        return R(f'{x.c}.{m.rname}({", ".join(s.args(m, args))})', norm(m.type))

    # ---- expressions -------------------------------------------------------------
    def expr(s, e, want=None, lhs=False):
        k = e.e
        if k == 'num': return s.num(e.v)
        if k == 'str':
            v = e.v
            return R(f'{json.dumps(eval_cs_string(v), ensure_ascii=False)}.to_string()', 'string')
        if k == 'chr': return R(e.v, 'char')
        if k == 'bool': return R(e.v, 'bool')
        if k == 'null': return R('null', 'null')
        if k == 'this': return R('self' if s.cls.kind != 'struct' else '(*self)', s.cls.name, s.cls.kind == 'struct')
        if k == 'paren':
            r = s.expr(e.x, want=want)
            return R(f'({r.c})', r.t, r.place and False)
        if k == 'name': return s.name(e.name)
        if k == 'typename': return R(e.name, 'type:' + e.name)
        if k == 'member': return s.member(e, lhs=lhs)
        if k == 'call': return s.call(e)
        if k == 'index': return s.index(e, lhs)
        if k == 'un': return s.unary(e, want)
        if k == 'bin': return s.bin(e, want)
        if k == 'cond':
            c = s.expr(e.c)
            a = s.expr(e.a, want=want); b = s.expr(e.b, want=want)
            t = want if (a.t == 'null' or b.t == 'null') and want else (b.t if a.t == 'null' else a.t if b.t == 'null' else s.promote(a.t, b.t))
            t = norm(t)
            if want and is_num(t) and is_num(want) and False: t = want
            return R(f'(if {c.c} {{ {s.conv(a, t)} }} else {{ {s.conv(b, t)} }})', t)
        if k == 'cast': return s.cast(e)
        if k == 'assign': return s.assign_expr(e)
        if k in ('post', 'pre'): return s.incdec(e)
        if k == 'new': return s.new(e)
        if k == 'newarray':
            sizes = [s.conv(s.expr(z), 'int') for z in e.sizes]
            if len(sizes) == 1: return R(f'Arr::new({sizes[0]})', e.base + '[]')
            return R(f'Arr2::new({sizes[0]}, {sizes[1]})', e.base + '[,]')
        if k == 'arrayinit':
            t = dict.get(e, 'type') or want
            if not t: raise KeyError('array init without type')
            return s.arrayinit(e, norm(t))
        if k == 'default': return R(f'<{rs(e.type)}>::default()', e.type)
        raise KeyError('expr ' + k)
    def arrayinit(s, e, t):
        et = elem(t)
        if t.endswith('[,]'):
            rows = [', '.join(s.conv(s.expr(i, want=et), et) for i in row['items']) for row in e['items']]
            return R('Arr2::from_rows(vec![' + ', '.join(f'vec![{r}]' for r in rows) + '])', t)
        items = [s.conv(s.expr(i, want=et), et) for i in e['items']]
        return R(f'Arr::from_vec(vec![{", ".join(items)}])', t)
    def num(s, v):
        lo = v.lower()
        if lo.startswith('0x'):
            d = lo.rstrip('ul')
            n = int(d, 16)
            if lo.endswith('ul') or lo.endswith('lu'): return R(d + 'u64', 'ulong')
            if lo.endswith('u'): return R(d + 'u32', 'uint')
            if lo.endswith('l'): return R(d + 'i64', 'long')
            if n > 0x7fffffff: return R(d + ('u32' if n <= 0xffffffff else 'u64'), 'uint' if n <= 0xffffffff else 'ulong')
            return R(d, 'int')
        if lo.endswith(('ul', 'lu')): return R(lo[:-2] + 'u64', 'ulong')
        if lo.endswith('u'): return R(lo[:-1] + 'u32', 'uint')
        if lo.endswith('l'): return R(lo[:-1] + 'i64', 'long')
        if lo.endswith('f'): return R(fl(lo[:-1]) + 'f32', 'float')
        if lo.endswith('d'): return R(fl(lo[:-1]), 'double')
        if '.' in lo or 'e' in lo: return R(fl(lo), 'double')
        return R(lo, 'int')
    def index(s, e, lhs):
        x = s.expr(e.x)
        t = norm(x.t)
        if t.startswith('Dictionary<'):
            k, v = split_args(t[11:-1])
            key = s.conv(s.expr(e.idx[0]), k)
            return R(f'{x.c}.get(&{key})', v, False) if not lhs else R(f'DICTLHS:{x.c}|{key}', v, True)
        et = elem(t)
        i = [s.conv(s.expr(z), 'int') for z in e.idx]
        if t == 'string': return R(f'rt::char_at(&{x.c}, {i[0]})', 'char')
        if len(i) == 2:
            if lhs: return R(f'(*{x.c}.at2({i[0]}, {i[1]}))', et, True)
            return R(f'{x.c}.at2({i[0]}, {i[1]}).clone()' if not s.tr.is_copy(et) else f'(*{x.c}.at2({i[0]}, {i[1]}))', et, False)
        return R(f'(*{x.c}.at({i[0]}))', et, True)
    def unary(s, e, want):
        x = s.expr(e.x, want=want)
        if e.op == '!': return R(f'!({x.c})', 'bool')
        if e.op == '+': return x
        t = x.t
        if t in ('byte', 'char', 'short'): t = 'int'; x = R(s.conv(x, 'int'), 'int')
        if e.op == '-':
            if t == 'uint': t = 'long'; x = R(s.conv(x, 'long'), 'long')
            return R(f'-{x.c}' if re.fullmatch(r'[\d.a-z_]+', x.c) else f'-({x.c})', t)
        if e.op == '~': return R(f'!({x.c})', t)
        raise KeyError(e.op)
    def bin(s, e, want):
        op = e.op
        if op == '??':
            ident = s.tr.static_ident(s.cls.name, e.l.name)
            return R(f'{ident}.with(|c__| {{ if c__.get().is_null() {{ c__.set({s.cls.name}::new()); }} c__.get() }})', s.cls.name)
        a = s.expr(e.l); b = s.expr(e.r)
        if op in ('&&', '||'): return R(f'({a.c} {op} {b.c})', 'bool')
        if op in ('==', '!='):
            neg = '!' if op == '!=' else ''
            if b.t == 'null' or a.t == 'null':
                y = a if b.t == 'null' else b
                if y.t.endswith('?'): return R(f'{neg}{y.c}.is_none()', 'bool')
                if y.t == 'string': return R(f'{neg}{y.c}.is_empty()', 'bool')
                return R(f'{neg}{y.c}.is_null()', 'bool')
            if a.t.endswith('?') and not b.t.endswith('?'):
                return R(f'({a.c} {op} Some({b.c}))', 'bool')
            if b.t.endswith('?') and not a.t.endswith('?'):
                return R(f'(Some({a.c}) {op} {b.c})', 'bool')
        if a.t == 'string' or b.t == 'string':
            if op == '+':
                return R(f'rt::cat({s.disp(a)}, {s.disp(b)})', 'string')
            return R(f'({a.c} {op} {b.c})', 'bool')
        if op in ('<<', '>>'):
            l = a
            lt = 'int' if a.t in ('byte', 'char', 'short') else a.t
            return R(f'({s.conv(l, lt)} {op} ({s.conv(b, "int")}))', lt)
        t = s.promote(a.t, b.t)
        if t in ('byte', 'short', 'char'): t = 'int'
        ca = s.conv(a, t); cb = s.conv(b, t)
        if a.t == 'bool' or a.t in s.tr.enums or a.t in s.tr.classes:
            ca, cb = a.c, b.c
        if op in ('==', '!=', '<', '>', '<=', '>='):
            return R(f'({ca} {op} {cb})', 'bool')
        return R(f'({ca} {op} {cb})', t)
    def disp(s, r):
        return r.c if r.t != 'string' else f'&{r.c}'
    def cast(s, e):
        x = s.expr(e.x, want=e.type if e.type.endswith(']') else None)
        t = norm(e.type)
        if t in RS and is_num(t):
            if x.t in s.tr.enums: return R(f'({x.c} as i32 as {RS[t]})', t)
            if x.t == 'char': return R(f'({x.c} as u32 as {RS[t]})', t)
            return R(f'(({x.c}) as {RS[t]})', t)
        if t in s.tr.enums:
            return R(f'{t}::from_i32({s.conv(x, "int")})', t)
        if t == 'char': return R(f'(({x.c}) as u8 as char)', 'char')
        return R(x.c, t, x.place)
    def incdec(s, e):
        op = '+' if e.op == '++' else '-'
        x = s.expr(e.x, lhs=True)
        one = '1.0' if x.t in ('double', 'float') else '1'
        if '.set_' in x.c: raise KeyError('incdec prop')
        if e.x.e == 'member' and x.c.endswith('()'):
            return s.assign_expr(N(e='assign', op=op + '=', l=e.x, r=N(e='num', v='1')))
        if e.e == 'pre': return R(f'{{ {x.c} {op}= {one}; {x.c} }}', x.t)
        return R(f'{{ let t__ = {x.c}; {x.c} {op}= {one}; t__ }}', x.t)
    def new(s, e):
        t = norm(e.type)
        m = re.match(r'(List|Queue|Stack)<', t)
        if m: return R('Arr::new(0)', t)
        if t.startswith('Dictionary<'): return R('Dict::new()', t)
        if t.startswith('KeyValuePair<'):
            k, v = split_args(t[13:-1])
            return R(f'Kv {{ key: {s.conv(s.expr(e.args[0]), k)}, value: {s.conv(s.expr(e.args[1]), v)} }}', t)
        c = s.tr.classes.get(t)
        if not c: raise KeyError('new ' + t)
        if not e.args and not c.ctors and c.kind == 'struct': return R(f'{t}::default()', t)
        if not e.args and not c.ctors: return R(f'{t}::new()', t)
        if not e.args and c.kind == 'struct' and not any(not m.params for m in c.ctors): return R(f'{t}::default()', t)
        m = s.tr.choose(c.ctors, s.argtypes(e.args))
        return R(f'{t}::{m.rname}({", ".join(s.args(m, e.args))})', t)

    def assign_expr(s, e):
        """Assignment used as an expression: { let v = rhs; lhs = v; v }."""
        code, t = s.assign(e)
        return R(code, t)

    def assign(s, e, stmt=False):
        l, op, rx = e.l, e.op, e.r
        # chain: a = b = c
        target = s.lhs_target(l)
        t = target[1]
        if op == '=':
            r = s.expr(rx, want=t)
            v = s.conv(r, t)
            if not stmt:
                return ('{ let v__ = ' + v + '; ' + s.store(target, 'v__.clone()') + '; v__ }', t)
            return (s.store(target, v), t)
        bop = op[:-1]
        cur = s.expr(l)
        r = s.expr(rx, want=t)
        if t == 'string':
            return (s.store(target, f'rt::cat(&{cur.c}, {s.disp(r)})'), t)
        pt = s.promote(cur.t, r.t) if bop not in ('<<', '>>') else cur.t
        if bop in ('<<', '>>'):
            val = f'({cur.c} {bop} ({s.conv(r, "int")}))'
        else:
            val = f'({s.conv(cur, pt)} {bop} {s.conv(r, pt)})'
        if pt != t and is_num(t): val = f'(({val}) as {RS[t]})'
        if target[0] == 'place' and pt == t and bop not in ('<<','>>'):
            return (f'{target[2]} {op} {s.conv(r, t)}', t)
        return (s.store(target, val), t)
    def lhs_target(s, l):
        if l.e == 'paren': return s.lhs_target(l.x)
        if l.e == 'index':
            x = s.expr(l.x)
            if norm(x.t).startswith('Dictionary<'):
                k, v = split_args(norm(x.t)[11:-1])
                return ('dict', v, x.c, s.conv(s.expr(l.idx[0]), k))
        r = s.expr(l, lhs=True)
        if r.c.endswith('()') and l.e in ('member', 'name'):
            # a property with a setter
            name = l.name
            if l.e == 'name':
                base = 'self'; c = s.cls
                while c and name not in c.props: c = getattr(c, 'outer', None)
            else:
                x = s.expr(l.x); base = x.c; c = s.tr.classes[norm(x.t)]
            return ('prop', r.t, f'{base}.{c.mname["S:" + name]}')
        return ('place', r.t, r.c)
    def store(s, target, v):
        if target[0] == 'place': return f'{target[2]} = {v}'
        if target[0] == 'prop': return f'{target[2]}({v})'
        if target[0] == 'dict': return f'{target[2]}.set({target[3]}, {v})'
        raise KeyError(target[0])

    # ---- statements ------------------------------------------------------------------
    def comments(s, st, out, ind):
        for c in dict.get(st, 'lead') or []: out.append(ind + cmt(c))
    def stmt(s, st, out, ind='    '):
        s.comments(st, out, ind)
        k = st.s
        before = len(out)
        s.stmt_(st, out, ind)
        tr = dict.get(st, 'trail') or []
        if tr and len(out) > before and k not in ('block', 'if', 'for', 'while', 'foreach', 'switch', 'try', 'do'):
            out[-1] += '  ' + ' '.join(cmt(c) for c in tr)
    def body(s, st, out, ind):
        s.push()
        if st.s == 'block':
            for x in st.body: s.stmt(x, out, ind)
            for c in dict.get(st, 'endlead') or []: out.append(ind + cmt(c))
        else: s.stmt(st, out, ind)
        s.pop()
    def stmt_(s, st, out, ind):
        k = st.s; deeper = ind + '    '
        if k == 'empty': return
        if k == 'block':
            out.append(ind + '{'); s.body(st, out, deeper); out.append(ind + '}'); return
        if k == 'expr': return s.expr_stmt(st.x, out, ind)
        if k == 'decl':
            for name, init in st.vars:
                t = norm(st.type)
                if init is None:
                    code = s.null_of(t) if t.endswith('?') or t == 'string' else f'<{rs(t)}>::default()'
                else:
                    code = s.conv(s.expr(init, want=t), t)
                s.local(name, t)
                out.append(f'{ind}let mut {snake(name)}: {rs(t)} = {code};')
            return
        if k == 'if':
            cur = st; prefix = 'if'
            while True:
                c = s.expr(cur.c).c
                out.append(f'{ind}{prefix} {strip_parens(c)} {{'); s.body(cur.a, out, deeper)
                if cur.b is None: out.append(ind + '}'); break
                if cur.b.s == 'if':
                    for cc in dict.get(cur.b, 'lead') or []: out.append(deeper + cmt(cc))
                    prefix = '} else if'; cur = cur.b; continue
                out.append(ind + '} else {'); s.body(cur.b, out, deeper); out.append(ind + '}'); break
            return
        if k == 'while':
            c = s.expr(st.c).c; lab = s.next_loop()
            out.append(f'{ind}{lab}: while {strip_parens(c)} {{')
            s.brk.append(('loop', lab, None)); s.body(st.body, out, deeper); s.brk.pop()
            out.append(ind + '}'); return
        if k == 'do':
            lab = s.next_loop()
            out.append(f'{ind}{lab}: loop {{')
            s.brk.append(('loop', lab, None)); s.body(st.body, out, deeper); s.brk.pop()
            out.append(f'{deeper}if !({s.expr(st.c).c}) {{ break; }}')
            out.append(ind + '}'); return
        if k == 'for':
            s.push()
            out.append(ind + '{')
            for i in st.init: s.stmt(i, out, deeper)
            c = s.expr(st.c).c if st.c else 'true'
            d2 = deeper + '    '
            hascont = contains_continue(st.body)
            lp = s.next_loop()
            out.append(f'{deeper}{lp}: while {strip_parens(c)} {{')
            if hascont:
                s.label += 1; lab = f"'c{s.label}"
                out.append(f'{d2}{lab}: {{')
                s.brk.append(('loop', lp, lab)); s.body(st.body, out, d2 + '    '); s.brk.pop()
                out.append(d2 + '}')
            else:
                s.brk.append(('loop', lp, None)); s.body(st.body, out, d2); s.brk.pop()
            for e in st.it: s.expr_stmt(e, out, d2)
            out.append(deeper + '}'); out.append(ind + '}'); s.pop(); return
        if k == 'foreach':
            coll = s.expr(st.coll)
            t = norm(st.type); s.push(); s.local(st.name, t)
            ct = norm(coll.t)
            lab = s.next_loop()
            out.append(f'{ind}{lab}: for mut {snake(st.name)} in {coll.c}.items() {{')
            s.brk.append(('loop', lab, None)); s.body(st.body, out, deeper); s.brk.pop()
            out.append(ind + '}'); s.pop(); return
        if k == 'return':
            if st.x is None: out.append(ind + 'return;'); return
            out.append(f'{ind}return {s.conv(s.expr(st.x, want=s.ret), s.ret)};'); return
        if k == 'break':
            if s.brk[-1][0] == 'closure': out.append(f'{ind}return true;')
            else: out.append(f'{ind}break {s.brk[-1][1]};')
            return
        if k == 'continue':
            for b in reversed(s.brk):
                if b[0] == 'loop':
                    out.append(f'{ind}break {b[2]};' if b[2] else f'{ind}continue {b[1]};'); return
            raise KeyError('continue')
        if k == 'throw': out.append(ind + 'panic!("thrown");'); return
        if k == 'switch': return s.switch(st, out, ind)
        if k == 'try':
            if st.catch is not None:
                # `return x` at the end of the body moves after the closure;
                # a `break` out of the body travels through the closure's result.
                body = st.body; tail = None
                if body.body and body.body[-1].s == 'return':
                    tail = body.body[-1]; body = N(s='block', body=body.body[:-1])
                broke = contains_break(body)
                if broke:
                    outer = s.brk[-1]
                    s.brk.append(('closure', None, None))
                    out.append(f'{ind}match rt::attempt_val(|| {{')
                    s.body(body, out, deeper)
                    s.brk.pop()
                    out.append(f'{deeper}false')
                    out.append(f'{ind}}}) {{')
                    out.append(f'{deeper}Some(true) => break {outer[1]},')
                    out.append(f'{deeper}Some(false) => {{}}')
                    out.append(f'{deeper}None => {{')
                    s.body(st.catch, out, deeper + '    ')
                    out.append(deeper + '}')
                    out.append(ind + '}')
                else:
                    out.append(f'{ind}if !rt::attempt(|| {{')
                    s.body(body, out, deeper)
                    out.append(f'{ind}}}) {{')
                    s.body(st.catch, out, deeper)
                    out.append(ind + '}')
                if tail is not None: s.stmt(tail, out, ind)
            else:
                s.body(st.body, out, ind)
            if st.fin: s.body(st.fin, out, ind)
            return
        raise KeyError('stmt ' + k)
    def next_loop(s):
        s.label += 1
        return f"'l{s.label}"
    def expr_stmt(s, x, out, ind):
        if x.e == 'assign':
            if x.r.e == 'assign' and x.op == '=':
                # a = b = c: b first, then a from b
                s.expr_stmt(x.r, out, ind)
                code, _ = s.assign(N(e='assign', op='=', l=x.l, r=x.r.l), stmt=True)
                out.append(f'{ind}{code};'); return
            code, _ = s.assign(x, stmt=True); out.append(f'{ind}{code};'); return
        if x.e in ('post', 'pre'):
            r = s.expr(x.x, lhs=True)
            op = '+=' if x.op == '++' else '-='
            if r.c.endswith('()'):
                code, _ = s.assign(N(e='assign', op=op, l=x.x, r=N(e='num', v='1')), stmt=True)
                out.append(f'{ind}{code};'); return
            one = '1.0' if r.t in ('double','float') else '1'
            out.append(f'{ind}{r.c} {op} {one};'); return
        out.append(f'{ind}{s.expr(x).c};')
    def switch(s, st, out, ind):
        x = s.expr(st.x); deeper = ind + '    '; d2 = deeper + '    '
        s.label += 1; lab = f"'sw{s.label}"
        subj = x.c if x.t != 'string' else f'{x.c}.as_str()'
        out.append(f'{ind}{lab}: {{'); out.append(f'{deeper}match {subj} {{')
        has_default = False
        for labels, body in st.cases:
            pats = []
            for l in labels:
                if l is None: pats.append('_'); has_default = True
                else:
                    p = s.expr(l).c
                    if x.t == 'string': p = p.replace('.to_string()', '')
                    pats.append(p)
            pat = '_' if '_' in pats else ' | '.join(pats)
            out.append(f'{d2}{pat} => {{')
            b = list(body)
            if len(b) == 1 and b[0].s == 'block': b = list(b[0].body) + [];
            while b and b[-1].s == 'break': b.pop()
            s.push(); s.brk.append(('switch', lab, None))
            for y in b: s.stmt(y, out, d2 + '    ')
            s.brk.pop(); s.pop()
            out.append(d2 + '}')
        if not has_default: out.append(f'{d2}_ => {{}}')
        out.append(deeper + '}'); out.append(ind + '}')


def contains_break(st):
    if isinstance(st, (list, tuple)): return any(contains_break(x) for x in st)
    if not isinstance(st, N): return False
    k = dict.get(st, 's')
    if k == 'break': return True
    if k in ('while', 'for', 'foreach', 'do', 'switch'): return False
    return any(contains_break(dict.get(st, key)) for key in ('body', 'a', 'b') if dict.get(st, key) is not None)

def contains_continue(st):
    if isinstance(st, (list, tuple)): return any(contains_continue(x) for x in st)
    if not isinstance(st, N): return False
    k = dict.get(st, 's')
    if k == 'continue': return True
    if k in ('while', 'for', 'foreach', 'do'): return False
    for key in ('body', 'a', 'b', 'catch', 'fin'):
        v = dict.get(st, key)
        if v is not None and contains_continue(v): return True
    if k == 'switch':
        return any(contains_continue(b) for _, b in st.cases)
    return False

def fl(s):
    if s.startswith('.'): s = '0' + s
    if s.endswith('.'): s += '0'
    if 'e' in s and '.' not in s: return s
    if '.' not in s and 'e' not in s: s += '.0'
    return s
def eval_cs_string(v):
    if v.startswith('@'): return v[2:-1].replace('""', '"')
    return json.loads(v) if '\\x' not in v else v[1:-1]
def strip_parens(c):
    if c.startswith('(') and c.endswith(')'):
        d = 0
        for i, ch in enumerate(c):
            if ch == '(': d += 1
            elif ch == ')':
                d -= 1
                if d == 0 and i < len(c) - 1: return c
        return c[1:-1]
    return c
def cmt(c):
    if c.startswith('/*'): return '// ' + ' '.join(c[2:-2].split())
    if c.startswith('///'):
        t = re.sub(r'</?(summary|param[^>]*|returns|remarks|para)>', '', c[3:])
        t = re.sub(r'<see cref="([^"]+)"\s*/>', r'`\1`', t)
        t = re.sub(r'<paramref name="([^"]+)"\s*/>', r'`\1`', t)
        return '//' + t if t.strip() else '//'
    return c

# ------------------------------------------------------------------ emit types
class Emit:
    def __init__(s, tr): s.tr = tr; s.files = {}
    def add(s, fname, text): s.files.setdefault(fname, []).append(text)
    def lead(s, d, ind=''):
        return ''.join(f'{ind}{cmt(c)}\n' for c in dict.get(d, 'lead') or [])
    def enum(s, d):
        o = s.lead(d)
        o += '#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]\n#[repr(i32)]\n'
        o += f'pub enum {d.name} {{\n'
        for i, (n, v, c, tr) in enumerate(d.vals):
            for x in c: o += f'    {cmt(x)}\n'
            if i == 0: o += '    #[default]\n'
            o += f'    {n}'
            o += ',' + ('  ' + ' '.join(cmt(x) for x in tr) if tr else '') + '\n'
        o += '}\n'
        arms = ''.join(f'{i} => {d.name}::{n}, ' for i, (n, _, _, _) in enumerate(d.vals))
        o += f'impl {d.name} {{ pub fn from_i32(v: i32) -> Self {{ match v {{ {arms} _ => Self::default() }} }} }}\n'
        o += f'impl std::fmt::Display for {d.name} {{ fn fmt(&self, f: &mut std::fmt::Formatter<\'_>) -> std::fmt::Result {{ write!(f, "{{:?}}", self) }} }}\n'
        s.add(d.file, o)

    def decl_order_members(s, c):
        ms = []
        for d, f in c.decls:
            ms.append((d, f))
        return ms

    def cls(s, c):
        tr = s.tr
        if c.extern or c.kind == 'interface': return
        struct = c.kind == 'struct'
        if 'static' in c.decls[0][0].mods and not c.fields and not c.props and False: pass
        data = c.name if struct else f'{c.name}Data'
        # struct
        o = ''
        d0 = c.decls[0][0]; f0 = c.decls[0][1]
        o += s.lead(d0)
        copy = tr.is_copy(c.name) if struct else True
        o += f'#[derive(Clone, Default{", Copy" if struct and copy else ""})]\npub struct {data} {{\n'
        def emit_field(n, m):
            r = ''
            for x in dict.get(m, 'lead') or []: r += f'    {cmt(x)}\n'
            tail = ' '.join(cmt(x) for x in (dict.get(m, 'trail') or []))
            r += f'    pub {c.fname[n]}: {rs(m.type)},' + ('  ' + tail if tail else '') + '\n'
            return r
        for n, m in c.fields.items(): o += emit_field(n, m)
        for n, m in c.props.items():
            if m.auto and 'static' not in m.mods: o += f'    pub {c.fname[n]}: {rs(m.type)},\n'
        o += '}\n'
        if not struct:
            o += f'rt::handle!({c.name}, {data});\n'
        s.add(f0, o)
        # statics: consts live in the first impl, the rest are module statics
        s.emit_impls(c)

    def impl_header(s, c): return f'impl {c.name} {{\n'

    def emit_impls(s, c):
        tr = s.tr; struct = c.kind == 'struct'
        # Which members were declared in which file (partial classes keep their files).
        own = []
        for d, f in c.decls:
            own.append((f, list(d.members)))
        base = getattr(c, 'basecls', None)
        if base is not None:
            own[0][1].extend([m for d, _ in base.decls for m in d.members
                              if not (m.node == 'ctor' or 'abstract' in m.mods or (m.node == 'field' and ('const' in m.mods or 'static' in m.mods)))
                              and not (m.node == 'method' and dict.get(m, 'rname') is None)
                              and not (m.node == 'method' and any(x is not m and len(x.params)==len(m.params) for x in c.methods.get(m.name, [])))])
        first = True
        for f, members in own:
            o = s.impl_header(c)
            if first:
                first = False
                if 'abstract' not in c.decls[0][0].mods: o += s.ctors(c)
            for m in members:
                try:
                    o += s.member(c, m)
                except KeyError as ex:
                    raise KeyError(f'{c.name}.{dict.get(m, "name")}: {ex}')
            o += '}\n'
            s.add(f, o)

    def alloc(s, c):
        """Field defaults, with the initialisers C# runs before a constructor."""
        struct = c.kind == 'struct'
        fn = Fn(s.tr, c, False, None)
        lines = []
        for n, m in c.fields.items():
            if m.init is not None:
                lines.append(f'this.{c.fname[n]} = {fn.conv(fn.expr(m.init, want=m.type), m.type)};'.replace('self.', 'this.'))
        return lines

    def ctors(s, c):
        struct = c.kind == 'struct'; o = ''
        data = c.name if struct else f'{c.name}Data'
        if struct:
            if not c.ctors: return ''
        lines = s.alloc(c)
        if not struct:
            o += f'    fn alloc__() -> {c.name} {{\n        let mut this = {c.name}::from_data({data}::default());\n'
            for l in lines: o += f'        {l}\n'
            o += '        this\n    }\n'
        if not c.ctors and not struct:
            o += f'    pub fn new() -> {c.name} {{ Self::alloc__() }}\n'
        for m in c.ctors:
            fn = Fn(s.tr, c, False, None)
            ps = s.params(fn, m)
            names = ', '.join(snake(p.name) for p in m.params)
            mk = f'let mut this = {c.name}::default();' if struct else 'let mut this = Self::alloc__();'
            o += f'    pub fn {m.rname}({ps}) -> {c.name} {{ {mk} this.{m.iname}({names}); this }}\n'
            recv = '&mut self' if struct else 'mut self'
            o += f'    fn {m.iname}({recv}{", " + ps if ps else ""}) {{\n'
            out = []
            if m.init:
                k, args = m.init
                tgt = s.tr.choose(c.ctors, fn.argtypes(args))
                out.append(f'        self.{tgt.iname}({", ".join(fn.args(tgt, args))});')
            if m.body: fn.body(m.body, out, '        ')
            o += '\n'.join(out) + '\n    }\n'
        return o

    def params(s, fn, m):
        out = []
        for p in m.params:
            t = norm(p.type); fn.local(p.name, t)
            if p.mod in ('ref', 'out'):
                fn.refparams.add(p.name)
                out.append(f'{snake(p.name)}: &mut {rs(t)}')
            else:
                out.append(f'mut {snake(p.name)}: {rs(t)}')
        return ', '.join(out)

    def member(s, c, m):
        tr = s.tr; struct = c.kind == 'struct'; k = m.node
        L = s.lead(m, '    ')
        if 'abstract' in c.decls[0][0].mods and 'static' not in m.mods and 'const' not in m.mods:
            return ''    # its instance members are copied into each subclass
        if k == 'field':
            if 'const' in m.mods:
                fn = Fn(tr, c, True, None)
                return L + f'    pub const {shout(m.name)}: {rs(m.type)} = {fn.conv(fn.expr(m.init, want=m.type), m.type)};\n'
            if 'static' in m.mods:
                s.static_field(c, m); return ''
            return ''
        if k == 'ctor': return ''
        if k == 'prop': return s.prop(c, m, L)
        if k == 'method':
            if m.body is None: return ''
            static = 'static' in m.mods
            ret = norm(m.type)
            fn = Fn(tr, c, static, ret)
            ps = s.params(fn, m)
            mutates = struct and not static and mutates_self(m.body, c)
            recv = '' if static else ('&mut self' if mutates else 'self') if struct else 'mut self'
            sig = ', '.join(x for x in (recv, ps) if x)
            rt_ = '' if ret == 'void' else f' -> {rs(ret)}'
            out = []
            fn.body(m.body, out, '        ')
            if ret != 'void' and not (m.body.body and m.body.body[-1].s in ('return', 'throw')):
                out.append('        #[allow(unreachable_code)]\n        unreachable!()')
            return L + f'    pub fn {m.rname}({sig}){rt_} {{\n' + '\n'.join(out) + '\n    }\n'
        raise KeyError(k)

    def prop(s, c, m, L):
        tr = s.tr; struct = c.kind == 'struct'; o = L
        t = norm(m.type); static = 'static' in m.mods
        recv = '' if static else 'self' if struct else 'mut self'
        if m.auto:
            f = c.fname[m.name]
            fn = Fn(tr, c, False, t)
            read = R(f'self.{f}', t, True)
            o += f'    pub fn {c.mname["P:" + m.name]}({recv}) -> {rs(t)} {{ {fn.mv(read)} }}\n'
            if m.set:
                r2 = '&mut self' if struct else 'mut self'
                o += f'    pub fn {c.mname["S:" + m.name]}({r2}, v: {rs(t)}) {{ self.{f} = v; }}\n'
            return o
        if m.get:
            fn = Fn(tr, c, static, t); out = []
            fn.body(m.get, out, '        ')
            if not (m.get.body and m.get.body[-1].s in ('return', 'throw')):
                out.append('        #[allow(unreachable_code)]\n        unreachable!()')
            o += f'    pub fn {c.mname["P:" + m.name]}({recv}) -> {rs(t)} {{\n' + '\n'.join(out) + '\n    }\n'
        if m.set:
            fn = Fn(tr, c, static, 'void'); fn.local('value', t); out = []
            fn.body(m.set, out, '        ')
            r2 = '' if static else ('&mut self, ' if struct else 'mut self, ')
            o += f'    pub fn {c.mname["S:" + m.name]}({r2}mut value: {rs(t)}) {{\n' + '\n'.join(out) + '\n    }\n'
        return o

    def static_field(s, c, m):
        tr = s.tr; ident = tr.static_ident(c.name, m.name)
        file = c.decls[0][1]
        if 'ThreadStatic' in (m.attrs or []):
            s.add(file, f'thread_local! {{ static {ident}: std::cell::Cell<{rs(m.type)}> = std::cell::Cell::new(Default::default()); }}' + '\n')
            return
        fn = Fn(tr, c, True, None)
        init = fn.conv(fn.expr(m.init, want=m.type), m.type) if m.init is not None else f'<{rs(m.type)}>::default()'
        s.add(file, s.lead(m) + f'pub static {ident}: rt::Global<{rs(m.type)}> = rt::Global::new(|| {init});\n')

def mutates_self(body, c):
    found = [False]
    def walk(x):
        if isinstance(x, (list, tuple)):
            for y in x: walk(y)
        elif isinstance(x, N):
            if dict.get(x, 'e') == 'assign':
                l = x.l
                while l.e in ('member', 'index', 'paren'): l = l.x
                if l.e == 'this' or (l.e == 'name' and l.name in c.fields): found[0] = True
            for v in dict.values(x): walk(v)
    walk(body); return found[0]

# ------------------------------------------------------------------ driver
FILES = ['Settings.cs'] + [f'Domain/{n}.cs' for n in 'Expression HotCorners Look Personality Pose Puff Troupe Walker WalkerAntics WalkerBrain WalkerEyes WalkerMind WalkerStyle'.split()] + \
        [f'Rendering/{n}.cs' for n in 'Surface SolidPainter MinionPainter BirdPainter FrameRenderer'.split()]

def main():
    src, dst = sys.argv[1], sys.argv[2]
    tr = Tr()
    for d in Parser(lex(PRELUDE)).file(): tr.add_type(d, 'prelude', extern=True)
    for f in FILES:
        text = open(os.path.join(src, f), encoding='utf-8-sig').read()
        fname = re.sub(r'(?<=[a-z0-9])([A-Z])', r'_\1', os.path.basename(f)[:-3]).lower()
        sub = os.path.dirname(f).lower()
        path = (sub + '/' if sub else '') + fname + '.rs'
        try:
            decls = Parser(lex(text)).file()
        except SyntaxError as ex:
            print('PARSE ERROR', f, ex); raise SystemExit(1)
        for d in decls: tr.add_type(d, path)
    # nested classes see their outer's statics
    for c in tr.classes.values():
        for d, _ in c.decls:
            for n in d.nested:
                if n.node != 'enum': tr.classes[n.name].outer = c
    tr.flatten()
    tr.assign_names()
    em = Emit(tr)
    errors = 0
    for e in tr.enums.values(): em.enum(e)
    for c in tr.classes.values():
        try:
            em.cls(c)
        except KeyError as ex:
            print('ERROR', c.name, ex); errors += 1
    mods = {}
    for path, parts in em.files.items():
        if path == 'prelude': continue
        full = os.path.join(dst, path); os.makedirs(os.path.dirname(full), exist_ok=True)
        # What C# left to the garbage collector.
        parts = [t.replace('self.puffs.remove_at(i);', 'puff.free(); self.puffs.remove_at(i);')
                  .replace('self.puffs.clear();', 'for p__ in self.puffs.items() { p__.free(); } self.puffs.clear();') for t in parts]
        hdr = '// Translated from the C# original by tools/cs2rs; keeps its statement order.\n#![allow(non_upper_case_globals)]\nuse crate::prelude::*;\n\n'
        with open(full, 'w', encoding='utf-8', newline='\n') as fh:
            fh.write(hdr + '\n'.join(parts))
        mods.setdefault(os.path.dirname(path), []).append(os.path.basename(path)[:-3])
    print('generated', sum(len(v) for v in mods.values()), 'files;', errors, 'errors')
    print(json.dumps(mods))
    return 1 if errors else 0

if __name__ == '__main__':
    sys.exit(main())
