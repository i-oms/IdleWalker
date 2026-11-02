// Seeded behaviour trace of the C# original; tools/sim_trace.rs is its twin.
// Built against the original's src/Domain and src/Settings.cs by tools/compare/compare.sh.
using System;
using System.Collections.Generic;
using System.Drawing;
using System.Globalization;
using System.Text;
using IdleWalker.Domain;

namespace IdleWalker.Tools
{
    sealed class TraceWorld : IWorld
    {
        public int Left { get { return 0; } }
        public int Right { get { return 3200; } }
        public int[] MonitorEdges { get { return new int[] { 0, 1920, 1920, 3200 }; } }
        public Rectangle MonitorBoundsAt(Point p) { return p.X < 1920 ? new Rectangle(0, 0, 1920, 1080) : new Rectangle(1920, 0, 1280, 800); }
        public double GroundAt(double x) { return x < 1920 ? 1040 : 760; }
        public int ScaleAt(double x) { return x < 1920 ? 3 : 4; }
    }

    static class SimTrace
    {
        static int Main()
        {
            Troupe troupe = new Troupe(new TraceWorld(), new Random(1234));
            troupe.EnsureCast();
            troupe.MinionMask = Troupe.AllMinionsMask;
            troupe.BirdMask = Troupe.AllBirdsMask;
            troupe.MinionsEnabled = true;
            troupe.BeanEnabled = true;
            troupe.BirdsEnabled = true;
            troupe.PantherEnabled = true;
            double dt = 1.0 / 60.0;
            int total = 60 * 600;
            ulong h = 14695981039346656037UL;
            for (int i = 0; i < total; i++)
            {
                Point cursor = new Point(600 + (i / 3000) * 300, 300);
                bool disturbed = i % 18000 == 17999;
                troupe.Step(dt, cursor, i > 10 && !disturbed, disturbed);
                foreach (Walker w in troupe.Walkers)
                {
                    Pose p = w.CurrentPose();
                    h = Mix(h, w.CentreX); h = Mix(h, w.FeetY); h = Mix(h, p.Yaw); h = Mix(h, p.ArmNear); h = Mix(h, p.Face.MouthCurve);
                }
                if (i % 600 == 0)
                {
                    StringBuilder sb = new StringBuilder();
                    sb.Append(i).Append(' ').Append(h.ToString("X16"));
                    foreach (Walker w in troupe.Walkers)
                        sb.Append(' ').Append(w.Personality.Name).Append('=').Append(w.Activity).Append('@').Append(((int)w.CentreX).ToString());
                    Console.WriteLine(sb.ToString());
                }
            }
            foreach (Walker w in troupe.Walkers)
                Console.WriteLine(w.Personality.Name + " exp=" + w.Experience() + " tastes=" + w.LearnedEntries() + " friends=" + w.Friendships());
            return 0;
        }

        static ulong Mix(ulong h, double d)
        {
            h ^= (ulong)BitConverter.DoubleToInt64Bits(d);
            return h * 1099511628211UL;
        }
    }
}
