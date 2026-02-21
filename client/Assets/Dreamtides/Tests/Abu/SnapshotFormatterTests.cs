#nullable enable

using System.Collections.Generic;
using NUnit.Framework;

namespace Abu.Tests
{
    public class SnapshotFormatterTests
    {
        [Test]
        public void FormatsASingleNonInteractiveNode()
        {
            var nodes = new List<AbuSceneNode>
            {
                new AbuSceneNode
                {
                    Role = "application",
                    Label = "Dreamtides",
                    Interactive = false,
                    Children = new List<AbuSceneNode>(),
                },
            };
            var result = SnapshotFormatter.Format(nodes, false);
            Assert.AreEqual("- application \"Dreamtides\"", result.Snapshot);
            Assert.AreEqual(0, result.Refs.Count);
        }

        [Test]
        public void FormatsASingleInteractiveNodeWithRef()
        {
            var nodes = new List<AbuSceneNode>
            {
                new AbuSceneNode
                {
                    Role = "button",
                    Label = "OK",
                    Interactive = true,
                    Children = new List<AbuSceneNode>(),
                },
            };
            var result = SnapshotFormatter.Format(nodes, false);
            Assert.AreEqual("- button \"OK\" [ref=e1]", result.Snapshot);
            Assert.AreEqual(1, result.Refs.Count);
            Assert.AreEqual("button", result.Refs["e1"].Role);
            Assert.AreEqual("OK", result.Refs["e1"].Name);
        }

        [Test]
        public void FormatsANodeWithNullLabel()
        {
            var nodes = new List<AbuSceneNode>
            {
                new AbuSceneNode
                {
                    Role = "group",
                    Label = null,
                    Interactive = false,
                    Children = new List<AbuSceneNode>(),
                },
            };
            var result = SnapshotFormatter.Format(nodes, false);
            Assert.AreEqual("- group", result.Snapshot);
            Assert.AreEqual(0, result.Refs.Count);
        }

        [Test]
        public void FormatsAnInteractiveNodeWithNullLabelUsingEmptyStringForName()
        {
            var nodes = new List<AbuSceneNode>
            {
                new AbuSceneNode
                {
                    Role = "button",
                    Label = null,
                    Interactive = true,
                    Children = new List<AbuSceneNode>(),
                },
            };
            var result = SnapshotFormatter.Format(nodes, false);
            Assert.AreEqual("- button [ref=e1]", result.Snapshot);
            Assert.AreEqual(1, result.Refs.Count);
            Assert.AreEqual("button", result.Refs["e1"].Role);
            Assert.AreEqual("", result.Refs["e1"].Name);
        }

        [Test]
        public void IndentsChildrenByTwoSpaces()
        {
            var nodes = new List<AbuSceneNode>
            {
                new AbuSceneNode
                {
                    Role = "application",
                    Label = "Dreamtides",
                    Interactive = false,
                    Children = new List<AbuSceneNode>
                    {
                        new AbuSceneNode
                        {
                            Role = "region",
                            Label = "UIToolkit",
                            Interactive = false,
                            Children = new List<AbuSceneNode>
                            {
                                new AbuSceneNode
                                {
                                    Role = "button",
                                    Label = "End Turn",
                                    Interactive = true,
                                    Children = new List<AbuSceneNode>(),
                                },
                            },
                        },
                    },
                },
            };
            var result = SnapshotFormatter.Format(nodes, false);
            var expected = string.Join(
                "\n",
                "- application \"Dreamtides\"",
                "  - region \"UIToolkit\"",
                "    - button \"End Turn\" [ref=e1]"
            );
            Assert.AreEqual(expected, result.Snapshot);
            Assert.AreEqual(1, result.Refs.Count);
            Assert.AreEqual("button", result.Refs["e1"].Role);
            Assert.AreEqual("End Turn", result.Refs["e1"].Name);
        }

        [Test]
        public void AssignsMonotonicallyIncrementingRefs()
        {
            var nodes = new List<AbuSceneNode>
            {
                new AbuSceneNode
                {
                    Role = "application",
                    Label = "Dreamtides",
                    Interactive = false,
                    Children = new List<AbuSceneNode>
                    {
                        new AbuSceneNode
                        {
                            Role = "region",
                            Label = "UIToolkit",
                            Interactive = false,
                            Children = new List<AbuSceneNode>
                            {
                                new AbuSceneNode
                                {
                                    Role = "button",
                                    Label = "End Turn",
                                    Interactive = true,
                                    Children = new List<AbuSceneNode>(),
                                },
                                new AbuSceneNode
                                {
                                    Role = "group",
                                    Label = "Hand",
                                    Interactive = false,
                                    Children = new List<AbuSceneNode>
                                    {
                                        new AbuSceneNode
                                        {
                                            Role = "button",
                                            Label = "Lightning Bolt",
                                            Interactive = true,
                                            Children = new List<AbuSceneNode>(),
                                        },
                                    },
                                },
                            },
                        },
                        new AbuSceneNode
                        {
                            Role = "region",
                            Label = "Scene3D",
                            Interactive = false,
                            Children = new List<AbuSceneNode>
                            {
                                new AbuSceneNode
                                {
                                    Role = "button",
                                    Label = "Undo",
                                    Interactive = true,
                                    Children = new List<AbuSceneNode>(),
                                },
                            },
                        },
                    },
                },
            };
            var result = SnapshotFormatter.Format(nodes, false);
            var expected = string.Join(
                "\n",
                "- application \"Dreamtides\"",
                "  - region \"UIToolkit\"",
                "    - button \"End Turn\" [ref=e1]",
                "    - group \"Hand\"",
                "      - button \"Lightning Bolt\" [ref=e2]",
                "  - region \"Scene3D\"",
                "    - button \"Undo\" [ref=e3]"
            );
            Assert.AreEqual(expected, result.Snapshot);
            Assert.AreEqual(3, result.Refs.Count);
            Assert.AreEqual("button", result.Refs["e1"].Role);
            Assert.AreEqual("End Turn", result.Refs["e1"].Name);
            Assert.AreEqual("button", result.Refs["e2"].Role);
            Assert.AreEqual("Lightning Bolt", result.Refs["e2"].Name);
            Assert.AreEqual("button", result.Refs["e3"].Role);
            Assert.AreEqual("Undo", result.Refs["e3"].Name);
        }

        [Test]
        public void HandlesMultipleLevelsOfNesting()
        {
            var nodes = new List<AbuSceneNode>
            {
                new AbuSceneNode
                {
                    Role = "application",
                    Label = "App",
                    Interactive = false,
                    Children = new List<AbuSceneNode>
                    {
                        new AbuSceneNode
                        {
                            Role = "region",
                            Label = "A",
                            Interactive = false,
                            Children = new List<AbuSceneNode>
                            {
                                new AbuSceneNode
                                {
                                    Role = "group",
                                    Label = "B",
                                    Interactive = false,
                                    Children = new List<AbuSceneNode>
                                    {
                                        new AbuSceneNode
                                        {
                                            Role = "group",
                                            Label = "C",
                                            Interactive = false,
                                            Children = new List<AbuSceneNode>
                                            {
                                                new AbuSceneNode
                                                {
                                                    Role = "button",
                                                    Label = "Deep",
                                                    Interactive = true,
                                                    Children = new List<AbuSceneNode>(),
                                                },
                                            },
                                        },
                                    },
                                },
                            },
                        },
                    },
                },
            };
            var result = SnapshotFormatter.Format(nodes, false);
            var expected = string.Join(
                "\n",
                "- application \"App\"",
                "  - region \"A\"",
                "    - group \"B\"",
                "      - group \"C\"",
                "        - button \"Deep\" [ref=e1]"
            );
            Assert.AreEqual(expected, result.Snapshot);
        }

        [Test]
        public void HandlesMultipleRootLevelNodes()
        {
            var nodes = new List<AbuSceneNode>
            {
                new AbuSceneNode
                {
                    Role = "region",
                    Label = "First",
                    Interactive = false,
                    Children = new List<AbuSceneNode>(),
                },
                new AbuSceneNode
                {
                    Role = "region",
                    Label = "Second",
                    Interactive = false,
                    Children = new List<AbuSceneNode>(),
                },
            };
            var result = SnapshotFormatter.Format(nodes, false);
            var expected = string.Join("\n", "- region \"First\"", "- region \"Second\"");
            Assert.AreEqual(expected, result.Snapshot);
        }

        [Test]
        public void TreatsEmptyStringLabelSameAsNull()
        {
            var nodes = new List<AbuSceneNode>
            {
                new AbuSceneNode
                {
                    Role = "group",
                    Label = "",
                    Interactive = false,
                    Children = new List<AbuSceneNode>(),
                },
            };
            var result = SnapshotFormatter.Format(nodes, false);
            Assert.AreEqual("- group", result.Snapshot);
        }

        [Test]
        public void CompactModeOmitsNonInteractiveLabellessNodesWithNoInteractiveDescendants()
        {
            var nodes = new List<AbuSceneNode>
            {
                new AbuSceneNode
                {
                    Role = "application",
                    Label = "App",
                    Interactive = false,
                    Children = new List<AbuSceneNode>
                    {
                        new AbuSceneNode
                        {
                            Role = "group",
                            Label = null,
                            Interactive = false,
                            Children = new List<AbuSceneNode>
                            {
                                new AbuSceneNode
                                {
                                    Role = "generic",
                                    Label = null,
                                    Interactive = false,
                                    Children = new List<AbuSceneNode>(),
                                },
                            },
                        },
                        new AbuSceneNode
                        {
                            Role = "group",
                            Label = null,
                            Interactive = false,
                            Children = new List<AbuSceneNode>
                            {
                                new AbuSceneNode
                                {
                                    Role = "button",
                                    Label = "Click Me",
                                    Interactive = true,
                                    Children = new List<AbuSceneNode>(),
                                },
                            },
                        },
                    },
                },
            };
            var result = SnapshotFormatter.Format(nodes, true);
            var expected = string.Join(
                "\n",
                "- application \"App\"",
                "  - group",
                "    - button \"Click Me\" [ref=e1]"
            );
            Assert.AreEqual(expected, result.Snapshot);
        }

        [Test]
        public void CompactModePreservesLabeledNonInteractiveNodes()
        {
            var nodes = new List<AbuSceneNode>
            {
                new AbuSceneNode
                {
                    Role = "region",
                    Label = "Info Panel",
                    Interactive = false,
                    Children = new List<AbuSceneNode>
                    {
                        new AbuSceneNode
                        {
                            Role = "generic",
                            Label = "Some Text",
                            Interactive = false,
                            Children = new List<AbuSceneNode>(),
                        },
                    },
                },
            };
            var result = SnapshotFormatter.Format(nodes, true);
            var expected = string.Join(
                "\n",
                "- region \"Info Panel\"",
                "  - generic \"Some Text\""
            );
            Assert.AreEqual(expected, result.Snapshot);
        }

        [Test]
        public void CompactModeIncludesNonInteractiveLabellessNodesWithInteractiveDescendants()
        {
            var nodes = new List<AbuSceneNode>
            {
                new AbuSceneNode
                {
                    Role = "group",
                    Label = null,
                    Interactive = false,
                    Children = new List<AbuSceneNode>
                    {
                        new AbuSceneNode
                        {
                            Role = "group",
                            Label = null,
                            Interactive = false,
                            Children = new List<AbuSceneNode>
                            {
                                new AbuSceneNode
                                {
                                    Role = "button",
                                    Label = "Nested",
                                    Interactive = true,
                                    Children = new List<AbuSceneNode>(),
                                },
                            },
                        },
                    },
                },
            };
            var result = SnapshotFormatter.Format(nodes, true);
            var expected = string.Join(
                "\n",
                "- group",
                "  - group",
                "    - button \"Nested\" [ref=e1]"
            );
            Assert.AreEqual(expected, result.Snapshot);
        }

        [Test]
        public void CompactFalseIncludesAllNodes()
        {
            var nodes = new List<AbuSceneNode>
            {
                new AbuSceneNode
                {
                    Role = "group",
                    Label = null,
                    Interactive = false,
                    Children = new List<AbuSceneNode>
                    {
                        new AbuSceneNode
                        {
                            Role = "generic",
                            Label = null,
                            Interactive = false,
                            Children = new List<AbuSceneNode>(),
                        },
                    },
                },
            };
            var result = SnapshotFormatter.Format(nodes, false);
            var expected = string.Join("\n", "- group", "  - generic");
            Assert.AreEqual(expected, result.Snapshot);
        }

        [Test]
        public void ReturnsEmptySnapshotForEmptyNodesArray()
        {
            var result = SnapshotFormatter.Format(new List<AbuSceneNode>(), false);
            Assert.AreEqual("", result.Snapshot);
            Assert.AreEqual(0, result.Refs.Count);
        }
    }
}
