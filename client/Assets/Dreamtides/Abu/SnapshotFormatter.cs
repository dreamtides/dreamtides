#nullable enable

using System.Collections.Generic;
using System.Text;

namespace Abu
{
    /// <summary>
    /// Converts a list of scene nodes into ARIA-style indented text with ref annotations.
    /// </summary>
    public static class SnapshotFormatter
    {
        /// <summary>
        /// Formats scene nodes into ARIA-style indented text.
        /// </summary>
        public static SnapshotData Format(List<AbuSceneNode> nodes, bool compact)
        {
            var lines = new List<string>();
            var refs = new Dictionary<string, SnapshotRef>();
            var refCounter = 0;

            foreach (var node in nodes)
            {
                Walk(node, 0, compact, lines, refs, ref refCounter);
            }

            return new SnapshotData
            {
                Snapshot = string.Join("\n", lines),
                Refs = refs,
            };
        }

        static void Walk(
            AbuSceneNode node,
            int depth,
            bool compact,
            List<string> lines,
            Dictionary<string, SnapshotRef> refs,
            ref int refCounter)
        {
            if (compact && !ShouldIncludeInCompact(node))
            {
                return;
            }

            var sb = new StringBuilder();
            sb.Append(new string(' ', depth * 2));
            sb.Append("- ");
            sb.Append(node.Role);

            if (!string.IsNullOrEmpty(node.Label))
            {
                sb.Append(" \"");
                sb.Append(node.Label);
                sb.Append('"');
            }

            if (node.Interactive)
            {
                refCounter++;
                var refStr = $"e{refCounter}";
                sb.Append($" [ref={refStr}]");
                refs[refStr] = new SnapshotRef
                {
                    Role = node.Role,
                    Name = node.Label ?? "",
                };
            }

            lines.Add(sb.ToString());

            foreach (var child in node.Children)
            {
                Walk(child, depth + 1, compact, lines, refs, ref refCounter);
            }
        }

        static bool ShouldIncludeInCompact(AbuSceneNode node)
        {
            if (node.Interactive)
            {
                return true;
            }

            if (!string.IsNullOrEmpty(node.Label))
            {
                return true;
            }

            return HasInteractiveDescendant(node);
        }

        static bool HasInteractiveDescendant(AbuSceneNode node)
        {
            foreach (var child in node.Children)
            {
                if (child.Interactive || HasInteractiveDescendant(child))
                {
                    return true;
                }
            }

            return false;
        }
    }
}
