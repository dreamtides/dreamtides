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

    /// <summary>
    /// Activates the BattleLayout.Contents game object to enable battle mode.
    /// </summary>
    void ActivateBattleContents()
    {
      Registry.BattleLayout.Contents.SetActive(true);
    }

    /// <summary>
    /// Deactivates the BattleLayout.Contents game object to disable battle
    /// mode and use the non-battle fallback.
    /// </summary>
    void DeactivateBattleContents()
    {
      Registry.BattleLayout.Contents.SetActive(false);
    }

    // -- Test 1: UI Toolkit basic walk (non-battle fallback) --

    [UnityTest]
    public IEnumerator UiToolkitBasicWalk_InteractiveElementAppearsInTree()
    {
      yield return Initialize();
      SetUpUIDocument();
      DeactivateBattleContents();

      var element = new NodeVisualElement { name = "TestButton" };
      element.pickingMode = PickingMode.Position;
      Registry.DocumentService.RootVisualElement.Add(element);

      var walker = CreateWalker();
      var refRegistry = new RefRegistry();
      var root = walker.Walk(refRegistry);

      Assert.AreEqual("application", root.Role);

      var uiToolkitRegion = root.Children.FirstOrDefault(c => c.Label == "UIToolkit");
      Assert.IsNotNull(uiToolkitRegion, "Should have a UIToolkit region");

      var found = FindNode(uiToolkitRegion!, n => n.Label == "TestButton" && n.Interactive);
      Assert.IsNotNull(found, "Interactive element should appear in the tree");
      Assert.AreEqual("button", found!.Role);
    }

    // -- Test 2: Non-interactive container recursion --

    [UnityTest]
    public IEnumerator NonInteractiveContainerRecursion_ChildFoundInGroup()
    {
      yield return Initialize();
      SetUpUIDocument();
      DeactivateBattleContents();

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

      var containerNode = FindNode(uiToolkitRegion!, n => n.Label == "Container");
      Assert.IsNotNull(containerNode, "Container should appear in tree");
      Assert.AreEqual("group", containerNode!.Role);
      Assert.IsFalse(containerNode.Interactive);

      var childNode = FindNode(uiToolkitRegion!, n => n.Label == "ChildButton" && n.Interactive);
      Assert.IsNotNull(childNode, "Interactive child should be found inside container");
      Assert.AreEqual("button", childNode!.Role);
    }

    // -- Test 3: Displayable discovery (non-battle fallback) --

    [UnityTest]
    public IEnumerator DisplayableDiscovery_InteractiveDisplayableAppearsInTree()
    {
      yield return Initialize();
      SetUpUIDocument();
      DeactivateBattleContents();

      CreateTestDisplayableButton("TestDisplayableBtn");

      var walker = CreateWalker();
      var refRegistry = new RefRegistry();
      var root = walker.Walk(refRegistry);

      var scene3dRegion = root.Children.FirstOrDefault(c => c.Label == "Scene3D");
      Assert.IsNotNull(scene3dRegion, "Should have a Scene3D region in non-battle mode");

      var found = FindNode(
        scene3dRegion!,
        n => n.Label == "TestDisplayableBtn" && n.Interactive
      );
      Assert.IsNotNull(found, "Interactive DisplayableButton should appear in tree");
      Assert.AreEqual("button", found!.Role);
    }

    // -- Test 4: Occlusion (battle mode, panels open) --

    [UnityTest]
    public IEnumerator Occlusion_3dContentOmittedWhenPanelsOpen()
    {
      yield return Initialize();
      SetUpUIDocument();
      ActivateBattleContents();

      Registry.DocumentService.HasOpenPanels = true;

      var element = new NodeVisualElement { name = "StillVisible" };
      element.pickingMode = PickingMode.Position;
      Registry.DocumentService.RootVisualElement.Add(element);

      var walker = CreateWalker();
      var refRegistry = new RefRegistry();
      var root = walker.Walk(refRegistry);

      var battleRegion = root.Children.FirstOrDefault(c => c.Label == "Battle");
      Assert.IsNotNull(battleRegion, "Should have a Battle region");

      // Controls should still be present
      var controlsGroup = FindNode(battleRegion!, n => n.Label == "Controls");
      Assert.IsNotNull(controlsGroup, "Controls should be present even when panels are open");

      // User and Opponent groups should NOT be present
      var userGroup = FindNode(battleRegion!, n => n.Label == "User");
      Assert.IsNull(userGroup, "User group should be omitted when HasOpenPanels is true");

      var opponentGroup = FindNode(battleRegion!, n => n.Label == "Opponent");
      Assert.IsNull(opponentGroup, "Opponent group should be omitted when HasOpenPanels is true");

      // UI Toolkit overlay should still be present with the element
      var uiToolkitRegion = FindNode(battleRegion!, n => n.Label == "UIToolkit");
      Assert.IsNotNull(uiToolkitRegion, "UIToolkit overlay should be present in battle mode");
      var visibleNode = FindNode(uiToolkitRegion!, n => n.Label == "StillVisible");
      Assert.IsNotNull(visibleNode, "UI Toolkit element should be present when panels are open");
    }

    // -- Test 5: Click dispatch --

    [UnityTest]
    public IEnumerator ClickDispatch_UiToolkitClickRecordsAction()
    {
      yield return Initialize();
      SetUpUIDocument();
      DeactivateBattleContents();

      var element = new NodeVisualElement { name = "ClickTarget" };
      element.pickingMode = PickingMode.Position;
      var callbacks = element.Callbacks.Value;
      var clickFired = false;
      callbacks.SetCallback(element, Callbacks.Event.Click, () => { clickFired = true; });

      Registry.DocumentService.RootVisualElement.Add(element);

      var walker = CreateWalker();
      var refRegistry = new RefRegistry();
      var root = walker.Walk(refRegistry);

      var targetNode = FindNode(root, n => n.Label == "ClickTarget" && n.Interactive);
      Assert.IsNotNull(targetNode, "Should find the clickable element");

      Assert.IsTrue(
        refRegistry.TryGetCallbacks("e1", out var refCallbacks),
        "Should have at least one ref registered"
      );

      refCallbacks.OnClick?.Invoke();

      Assert.IsTrue(clickFired, "Click callback should have been invoked via ref dispatch");
    }

    // -- Test 6: Tree structure (battle mode) --

    [UnityTest]
    public IEnumerator TreeStructure_BattleModeHasExpectedGroups()
    {
      yield return Initialize();
      SetUpUIDocument();
      ActivateBattleContents();
      Registry.DocumentService.HasOpenPanels = false;

      var walker = CreateWalker();
      var refRegistry = new RefRegistry();
      var root = walker.Walk(refRegistry);

      Assert.AreEqual("application", root.Role);
      Assert.AreEqual("Dreamtides", root.Label);

      var battleRegion = root.Children.FirstOrDefault(c => c.Label == "Battle");
      Assert.IsNotNull(battleRegion, "Should have a Battle region");
      Assert.AreEqual("region", battleRegion!.Role);

      var controlsGroup = FindNode(battleRegion, n => n.Label == "Controls");
      Assert.IsNotNull(controlsGroup, "Should have Controls group");

      var userGroup = FindNode(battleRegion, n => n.Label == "User");
      Assert.IsNotNull(userGroup, "Should have User group");

      var opponentGroup = FindNode(battleRegion, n => n.Label == "Opponent");
      Assert.IsNotNull(opponentGroup, "Should have Opponent group");

      var actionsGroup = FindNode(battleRegion, n => n.Label == "Actions");
      Assert.IsNotNull(actionsGroup, "Should have Actions group");

      // User group should have Status subgroup
      var userStatus = FindNode(userGroup!, n => n.Label == "Status");
      Assert.IsNotNull(userStatus, "User should have Status subgroup");

      // Opponent group should have Status subgroup
      var opponentStatus = FindNode(opponentGroup!, n => n.Label == "Status");
      Assert.IsNotNull(opponentStatus, "Opponent should have Status subgroup");
    }

    // -- Test 7: Tree structure (non-battle fallback) --

    [UnityTest]
    public IEnumerator TreeStructure_NonBattleHasUiToolkitAndScene3D()
    {
      yield return Initialize();
      SetUpUIDocument();
      DeactivateBattleContents();

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

    // -- Test 8: Hover callback registration --

    [UnityTest]
    public IEnumerator HoverCallback_RegisteredForInteractiveElements()
    {
      yield return Initialize();
      SetUpUIDocument();
      DeactivateBattleContents();

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

    // -- Test 9: Battle mode card in battlefield --

    [UnityTest]
    public IEnumerator BattleMode_CardInBattlefieldAppearsInUserGroup()
    {
      yield return Initialize();
      SetUpUIDocument();
      ActivateBattleContents();
      Registry.DocumentService.HasOpenPanels = false;

      var card = CreateTestCard();
      card.GameContext = Layout.GameContext.Battlefield;
      card._cardView.Revealed = new Schema.RevealedCardView
      {
        Name = "Test Character",
        CardType = "Character",
        Spark = "5",
      };
      Registry.BattleLayout.UserBattlefield.Add(card);

      var walker = CreateWalker();
      var refRegistry = new RefRegistry();
      var root = walker.Walk(refRegistry);

      var battleRegion = FindNode(root, n => n.Label == "Battle");
      Assert.IsNotNull(battleRegion);
      var userGroup = FindNode(battleRegion!, n => n.Label == "User");
      Assert.IsNotNull(userGroup);
      var battlefield = FindNode(userGroup!, n => n.Label == "Battlefield");
      Assert.IsNotNull(battlefield);

      var cardNode = FindNode(
        battlefield!,
        n => n.Label != null && n.Label.Contains("Test Character")
      );
      Assert.IsNotNull(cardNode, "Card should appear in User's Battlefield group");
      Assert.IsTrue(cardNode!.Interactive);
      Assert.IsTrue(
        cardNode.Label!.Contains("spark: 5"),
        "Battlefield card should show spark annotation"
      );
    }

    // -- Test 10: StripRichText handles tags and PUA characters --

    [UnityTest]
    public IEnumerator StripRichText_HandlesRichTextAndPua()
    {
      yield return Initialize();
      SetUpUIDocument();
      ActivateBattleContents();
      Registry.DocumentService.HasOpenPanels = false;

      // Set up energy with rich text tags and PUA icon
      var status = Registry.BattleLayout.UserStatusDisplay;
      status._energy._originalText = "3/7<color=#00838F>\ufa1f</color>";
      status._score._originalText = "2<size=80%>\ufb43</size>";
      status._totalSpark._originalText = "49";

      var walker = CreateWalker();
      var refRegistry = new RefRegistry();
      var root = walker.Walk(refRegistry);

      var battleRegion = FindNode(root, n => n.Label == "Battle");
      Assert.IsNotNull(battleRegion);
      var userGroup = FindNode(battleRegion!, n => n.Label == "User");
      Assert.IsNotNull(userGroup);
      var statusGroup = FindNode(userGroup!, n => n.Label == "Status");
      Assert.IsNotNull(statusGroup);

      var energyLabel = FindNode(statusGroup!, n => n.Label != null && n.Label.StartsWith("Energy:"));
      Assert.IsNotNull(energyLabel, "Should have energy label");
      Assert.AreEqual("Energy: 3/7", energyLabel!.Label);

      var scoreLabel = FindNode(statusGroup!, n => n.Label != null && n.Label.StartsWith("Score:"));
      Assert.IsNotNull(scoreLabel, "Should have score label");
      Assert.AreEqual("Score: 2", scoreLabel!.Label);

      var sparkLabel = FindNode(statusGroup!, n => n.Label != null && n.Label.StartsWith("Spark:"));
      Assert.IsNotNull(sparkLabel, "Should have spark label");
      Assert.AreEqual("Spark: 49", sparkLabel!.Label);
    }

    // -- Helper methods --

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
