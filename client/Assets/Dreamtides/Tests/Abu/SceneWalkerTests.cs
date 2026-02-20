#nullable enable

using System.Collections;
using System.Linq;
using Abu;
using Dreamtides.Abu;
using Dreamtides.Buttons;
using Dreamtides.Masonry;
using Dreamtides.Tests.TestUtils;
using NUnit.Framework;
using TMPro;
using UnityEngine;
using UnityEngine.TestTools;
using UnityEngine.UIElements;

namespace Dreamtides.Tests.Abu
{
  [TestFixture]
  public class SceneWalkerTests : DreamtidesUnitTest
  {
    DreamtidesSceneWalker CreateWalker()
    {
      return new DreamtidesSceneWalker(Registry);
    }

    /// <summary>
    /// Attaches a UIDocument with PanelSettings to the DocumentService so that
    /// RootVisualElement is available in tests.
    /// </summary>
    void SetUpUIDocument()
    {
      var panelSettings = ScriptableObject.CreateInstance<PanelSettings>();
      var docGo = new GameObject("UIDocumentHost");
      var uiDocument = docGo.AddComponent<UIDocument>();
      uiDocument.panelSettings = panelSettings;
      Registry.DocumentService._document = uiDocument;
    }

    // -- Test 1: UI Toolkit basic walk --

    /// <summary>
    /// Add a VisualElement with pickingMode = Position to
    /// DocumentService.RootVisualElement. Walk. Verify the element appears as
    /// interactive in the output tree.
    /// </summary>
    [UnityTest]
    public IEnumerator UiToolkitBasicWalk_InteractiveElementAppearsInTree()
    {
      yield return Initialize();
      SetUpUIDocument();

      var element = new NodeVisualElement { name = "TestButton" };
      element.pickingMode = PickingMode.Position;
      Registry.DocumentService.RootVisualElement.Add(element);

      var walker = CreateWalker();
      var refRegistry = new RefRegistry();
      var root = walker.Walk(refRegistry);

      Assert.AreEqual("application", root.Role);

      // Find UIToolkit region
      var uiToolkitRegion = root.Children.FirstOrDefault(c => c.Label == "UIToolkit");
      Assert.IsNotNull(uiToolkitRegion, "Should have a UIToolkit region");

      // Find the interactive element somewhere in the tree
      var found = FindNode(uiToolkitRegion!, n => n.Label == "TestButton" && n.Interactive);
      Assert.IsNotNull(found, "Interactive element should appear in the tree");
      Assert.AreEqual("button", found!.Role);
    }

    // -- Test 2: Non-interactive container recursion --

    /// <summary>
    /// Add a container with pickingMode = Ignore containing an interactive child.
    /// Walk. Verify the container is "group" role and child is found.
    /// </summary>
    [UnityTest]
    public IEnumerator NonInteractiveContainerRecursion_ChildFoundInGroup()
    {
      yield return Initialize();
      SetUpUIDocument();

      var container = new NodeVisualElement { name = "Container" };
      container.pickingMode = PickingMode.Ignore;

      var child = new NodeVisualElement { name = "ChildButton" };
      child.pickingMode = PickingMode.Position;
      container.Add(child);

      Registry.DocumentService.RootVisualElement.Add(container);

      var walker = CreateWalker();
      var refRegistry = new RefRegistry();
      var root = walker.Walk(refRegistry);

      var uiToolkitRegion = root.Children.FirstOrDefault(c => c.Label == "UIToolkit");
      Assert.IsNotNull(uiToolkitRegion);

      // Find container - should be group
      var containerNode = FindNode(uiToolkitRegion!, n => n.Label == "Container");
      Assert.IsNotNull(containerNode, "Container should appear in tree");
      Assert.AreEqual("group", containerNode!.Role);
      Assert.IsFalse(containerNode.Interactive);

      // Find child inside - should be interactive button
      var childNode = FindNode(uiToolkitRegion!, n => n.Label == "ChildButton" && n.Interactive);
      Assert.IsNotNull(childNode, "Interactive child should be found inside container");
      Assert.AreEqual("button", childNode!.Role);
    }

    // -- Test 3: Displayable discovery --

    /// <summary>
    /// Create an interactive Displayable. Walk. Verify it appears in the output.
    /// </summary>
    [UnityTest]
    public IEnumerator DisplayableDiscovery_InteractiveDisplayableAppearsInTree()
    {
      yield return Initialize();
      SetUpUIDocument();

      Registry.DocumentService.HasOpenPanels = false;

      CreateTestDisplayableButton("TestDisplayableBtn");

      var walker = CreateWalker();
      var refRegistry = new RefRegistry();
      var root = walker.Walk(refRegistry);

      var scene3dRegion = root.Children.FirstOrDefault(c => c.Label == "Scene3D");
      Assert.IsNotNull(scene3dRegion, "Should have a Scene3D region");

      var found = FindNode(
        scene3dRegion!,
        n => n.Label == "TestDisplayableBtn" && n.Interactive
      );
      Assert.IsNotNull(found, "Interactive DisplayableButton should appear in tree");
      Assert.AreEqual("button", found!.Role);
    }

    // -- Test 4: Occlusion --

    /// <summary>
    /// Set HasOpenPanels = true. Create a Displayable. Walk. Verify Displayable
    /// is NOT in output. UI Toolkit elements should still appear.
    /// </summary>
    [UnityTest]
    public IEnumerator Occlusion_DisplayablesOmittedWhenPanelsOpen()
    {
      yield return Initialize();
      SetUpUIDocument();

      Registry.DocumentService.HasOpenPanels = true;

      CreateTestDisplayableButton("OccludedButton");

      // Add a UI Toolkit element that should still appear
      var uiElement = new NodeVisualElement { name = "StillVisible" };
      uiElement.pickingMode = PickingMode.Position;
      Registry.DocumentService.RootVisualElement.Add(uiElement);

      var walker = CreateWalker();
      var refRegistry = new RefRegistry();
      var root = walker.Walk(refRegistry);

      var scene3dRegion = root.Children.FirstOrDefault(c => c.Label == "Scene3D");
      Assert.IsNotNull(scene3dRegion);

      // Displayable should NOT be in the Scene3D region
      var occludedNode = FindNode(scene3dRegion!, n => n.Label == "OccludedButton");
      Assert.IsNull(occludedNode, "Displayable should be omitted when HasOpenPanels is true");

      // UI Toolkit element should still be present
      var uiToolkitRegion = root.Children.FirstOrDefault(c => c.Label == "UIToolkit");
      Assert.IsNotNull(uiToolkitRegion);
      var visibleNode = FindNode(uiToolkitRegion!, n => n.Label == "StillVisible");
      Assert.IsNotNull(
        visibleNode,
        "UI Toolkit element should still be in tree when panels are open"
      );
    }

    // -- Test 5: Click dispatch --

    /// <summary>
    /// Walk to get refs. Dispatch a click to a UI Toolkit ref. Verify the action
    /// was recorded.
    /// </summary>
    [UnityTest]
    public IEnumerator ClickDispatch_UiToolkitClickRecordsAction()
    {
      yield return Initialize();
      SetUpUIDocument();

      var element = new NodeVisualElement { name = "ClickTarget" };
      element.pickingMode = PickingMode.Position;
      var callbacks = element.Callbacks.Value;
      var clickFired = false;
      callbacks.SetCallback(element, Callbacks.Event.Click, () => { clickFired = true; });

      Registry.DocumentService.RootVisualElement.Add(element);

      var walker = CreateWalker();
      var refRegistry = new RefRegistry();
      var root = walker.Walk(refRegistry);

      // Find the interactive element and its ref
      var targetNode = FindNode(root, n => n.Label == "ClickTarget" && n.Interactive);
      Assert.IsNotNull(targetNode, "Should find the clickable element");

      // Find the ref string for this element -- it should be "e1" or similar
      // We iterate the registry to find any registered callback and invoke it
      Assert.IsTrue(
        refRegistry.TryGetCallbacks("e1", out var refCallbacks),
        "Should have at least one ref registered"
      );

      refCallbacks.OnClick?.Invoke();

      Assert.IsTrue(clickFired, "Click callback should have been invoked via ref dispatch");
    }

    // -- Test 6: Tree structure --

    /// <summary>
    /// Verify the snapshot tree has the expected structure with application root
    /// and two region children.
    /// </summary>
    [UnityTest]
    public IEnumerator TreeStructure_HasApplicationRootWithRegions()
    {
      yield return Initialize();
      SetUpUIDocument();

      var walker = CreateWalker();
      var refRegistry = new RefRegistry();
      var root = walker.Walk(refRegistry);

      Assert.AreEqual("application", root.Role);
      Assert.AreEqual("Dreamtides", root.Label);
      Assert.AreEqual(2, root.Children.Count, "Should have UIToolkit and Scene3D regions");
      Assert.AreEqual("region", root.Children[0].Role);
      Assert.AreEqual("UIToolkit", root.Children[0].Label);
      Assert.AreEqual("region", root.Children[1].Role);
      Assert.AreEqual("Scene3D", root.Children[1].Label);
    }

    // -- Test 7: Hover callback registration --

    /// <summary>
    /// Verify that hover callbacks are registered for interactive UI Toolkit
    /// elements.
    /// </summary>
    [UnityTest]
    public IEnumerator HoverCallback_RegisteredForInteractiveElements()
    {
      yield return Initialize();
      SetUpUIDocument();

      var element = new NodeVisualElement { name = "HoverTarget" };
      element.pickingMode = PickingMode.Position;
      var callbacks = element.Callbacks.Value;
      var hoverFired = false;
      callbacks.SetCallback(
        element,
        Callbacks.Event.MouseEnter,
        () => { hoverFired = true; }
      );

      Registry.DocumentService.RootVisualElement.Add(element);

      var walker = CreateWalker();
      var refRegistry = new RefRegistry();
      walker.Walk(refRegistry);

      Assert.IsTrue(
        refRegistry.TryGetCallbacks("e1", out var refCallbacks),
        "Should have a ref for the hoverable element"
      );

      Assert.IsNotNull(refCallbacks.OnHover, "Hover callback should be registered");
      refCallbacks.OnHover!.Invoke();

      Assert.IsTrue(hoverFired, "Hover callback should have been invoked");
    }

    // -- Helper methods --

    /// <summary>
    /// Creates a fully configured DisplayableButton for use in tests.
    /// </summary>
    DisplayableButton CreateTestDisplayableButton(string label)
    {
      return CreateSceneObject<DisplayableButton>(b =>
      {
        b._background = b.gameObject.AddComponent<SpriteRenderer>();
        var textGo = new GameObject("ButtonText");
        textGo.transform.SetParent(b.transform);
        b._text = textGo.AddComponent<TextMeshPro>();
        b._text.text = label;
        var colliderGo = new GameObject("ButtonCollider");
        colliderGo.transform.SetParent(b.transform);
        b._collider = colliderGo.AddComponent<BoxCollider>();
        b._noOutlineMaterial = new Material(Shader.Find("Sprites/Default"));
      });
    }

    /// <summary>
    /// Recursively search the scene tree for a node matching the predicate.
    /// </summary>
    static AbuSceneNode? FindNode(AbuSceneNode node, System.Func<AbuSceneNode, bool> predicate)
    {
      if (predicate(node))
      {
        return node;
      }

      foreach (var child in node.Children)
      {
        var found = FindNode(child, predicate);
        if (found != null)
        {
          return found;
        }
      }

      return null;
    }
  }
}
