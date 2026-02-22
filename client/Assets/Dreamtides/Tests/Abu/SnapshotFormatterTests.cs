#nullable enable

using System.Collections.Generic;
using NUnit.Framework;

namespace Abu.Tests
{
  public class SnapshotFormatterTests
  {
    static AbuSceneNode Node(
      string role,
      string? label = null,
      bool interactive = false,
      params AbuSceneNode[] children
    ) =>
      new AbuSceneNode
      {
        Role = role,
        Label = label,
        Interactive = interactive,
        Children = new List<AbuSceneNode>(children),
      };

    [Test]
    public void FormatsASingleNonInteractiveNode()
    {
      var nodes = new List<AbuSceneNode> { Node("application", "Dreamtides") };
      var result = SnapshotFormatter.Format(nodes, false);
      Assert.AreEqual("- application \"Dreamtides\"", result.Snapshot);
      Assert.AreEqual(0, result.Refs.Count);
    }

    [Test]
    public void FormatsASingleInteractiveNodeWithRef()
    {
      var nodes = new List<AbuSceneNode> { Node("button", "OK", true) };
      var result = SnapshotFormatter.Format(nodes, false);
      Assert.AreEqual("- button \"OK\" [ref=e1]", result.Snapshot);
      Assert.AreEqual(1, result.Refs.Count);
      Assert.AreEqual("button", result.Refs["e1"].Role);
      Assert.AreEqual("OK", result.Refs["e1"].Name);
    }

    [Test]
    public void FormatsANodeWithNullLabel()
    {
      var nodes = new List<AbuSceneNode> { Node("group") };
      var result = SnapshotFormatter.Format(nodes, false);
      Assert.AreEqual("- group", result.Snapshot);
      Assert.AreEqual(0, result.Refs.Count);
    }

    [Test]
    public void FormatsAnInteractiveNodeWithNullLabelUsingEmptyStringForName()
    {
      var nodes = new List<AbuSceneNode> { Node("button", null, true) };
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
        Node(
          "application",
          "Dreamtides",
          false,
          Node("region", "UIToolkit", false, Node("button", "End Turn", true))
        ),
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
        Node(
          "application",
          "Dreamtides",
          false,
          Node(
            "region",
            "UIToolkit",
            false,
            Node("button", "End Turn", true),
            Node("group", "Hand", false, Node("button", "Lightning Bolt", true))
          ),
          Node("region", "Scene3D", false, Node("button", "Undo", true))
        ),
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
        Node(
          "application",
          "App",
          false,
          Node(
            "region",
            "A",
            false,
            Node("group", "B", false, Node("group", "C", false, Node("button", "Deep", true)))
          )
        ),
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
      var nodes = new List<AbuSceneNode> { Node("region", "First"), Node("region", "Second") };
      var result = SnapshotFormatter.Format(nodes, false);
      var expected = string.Join("\n", "- region \"First\"", "- region \"Second\"");
      Assert.AreEqual(expected, result.Snapshot);
    }

    [Test]
    public void TreatsEmptyStringLabelSameAsNull()
    {
      var nodes = new List<AbuSceneNode> { Node("group", "") };
      var result = SnapshotFormatter.Format(nodes, false);
      Assert.AreEqual("- group", result.Snapshot);
    }

    [Test]
    public void CompactModeOmitsNonInteractiveLabellessNodesWithNoInteractiveDescendants()
    {
      var nodes = new List<AbuSceneNode>
      {
        Node(
          "application",
          "App",
          false,
          Node("group", null, false, Node("generic")),
          Node("group", null, false, Node("button", "Click Me", true))
        ),
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
        Node("region", "Info Panel", false, Node("generic", "Some Text")),
      };
      var result = SnapshotFormatter.Format(nodes, true);
      var expected = string.Join("\n", "- region \"Info Panel\"", "  - generic \"Some Text\"");
      Assert.AreEqual(expected, result.Snapshot);
    }

    [Test]
    public void CompactModeIncludesNonInteractiveLabellessNodesWithInteractiveDescendants()
    {
      var nodes = new List<AbuSceneNode>
      {
        Node("group", null, false, Node("group", null, false, Node("button", "Nested", true))),
      };
      var result = SnapshotFormatter.Format(nodes, true);
      var expected = string.Join("\n", "- group", "  - group", "    - button \"Nested\" [ref=e1]");
      Assert.AreEqual(expected, result.Snapshot);
    }

    [Test]
    public void CompactFalseIncludesAllNodes()
    {
      var nodes = new List<AbuSceneNode> { Node("group", null, false, Node("generic")) };
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
