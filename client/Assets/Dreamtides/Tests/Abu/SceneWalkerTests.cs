#nullable enable

using System.Collections;
using System.Linq;
using Abu;
using Dreamtides.Abu;
using Dreamtides.Buttons;
using Dreamtides.Components;
using Dreamtides.Layout;
using Dreamtides.Masonry;
using Dreamtides.Schema;
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

    // -- Test 1: UI Toolkit basic walk (non-battle fallback) --

    /// <summary>
    /// Add a VisualElement with pickingMode = Position to
    /// DocumentService.RootVisualElement. Walk in non-battle mode. Verify
    /// the element appears as interactive in the output tree.
    /// </summary>
    [UnityTest]
    public IEnumerator UiToolkitBasicWalk_InteractiveElementAppearsInTree()
    {
      yield return Initialize();
      SetUpUIDocument();

      // Ensure we're not in battle mode
      Registry.BattleLayout.Contents.SetActive(false);

      var element = new NodeVisualElement { name = "TestButton" };
      element.pickingMode = PickingMode.Position;
      Registry.DocumentService.RootVisualElement.Add(element);

      var walker = CreateWalker();
      var refRegistry = new RefRegistry();
      var root = walker.Walk(refRegistry);

      Assert.AreEqual("application", root.Role);

      // Find Quest region, then UIToolkit nested inside it
      var questRegion = root.Children.FirstOrDefault(c => c.Label == "Quest");
      Assert.IsNotNull(questRegion, "Should have a Quest region");
      var uiToolkitRegion = FindNode(questRegion!, n => n.Label == "UIToolkit");
      Assert.IsNotNull(uiToolkitRegion, "Should have a UIToolkit region inside Quest");

      // Find the interactive element somewhere in the tree
      var found = FindNode(uiToolkitRegion!, n => n.Label == "TestButton" && n.Interactive);
      Assert.IsNotNull(found, "Interactive element should appear in the tree");
      Assert.AreEqual("button", found!.Role);
    }

    // -- Test 2: Non-interactive container recursion --

    /// <summary>
    /// Add a container with pickingMode = Ignore containing an interactive child.
    /// Walk in non-battle mode. Verify the container is "group" role and child is found.
    /// </summary>
    [UnityTest]
    public IEnumerator NonInteractiveContainerRecursion_ChildFoundInGroup()
    {
      yield return Initialize();
      SetUpUIDocument();

      Registry.BattleLayout.Contents.SetActive(false);

      var container = new NodeVisualElement { name = "Container" };
      container.pickingMode = PickingMode.Ignore;

      var child = new NodeVisualElement { name = "ChildButton" };
      child.pickingMode = PickingMode.Position;
      container.Add(child);

      Registry.DocumentService.RootVisualElement.Add(container);

      var walker = CreateWalker();
      var refRegistry = new RefRegistry();
      var root = walker.Walk(refRegistry);

      var questRegion = root.Children.FirstOrDefault(c => c.Label == "Quest");
      Assert.IsNotNull(questRegion);
      var uiToolkitRegion = FindNode(questRegion!, n => n.Label == "UIToolkit");
      Assert.IsNotNull(uiToolkitRegion, "Should have a UIToolkit region inside Quest");

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

    // -- Test 3: Quest mode produces Quest region, not Scene3D --

    /// <summary>
    /// Walk in non-battle mode. Verify quest mode produces a Quest region
    /// and does not produce a Scene3D region.
    /// </summary>
    [UnityTest]
    public IEnumerator QuestMode_ProducesQuestRegionNotScene3D()
    {
      yield return Initialize();
      SetUpUIDocument();

      Registry.BattleLayout.Contents.SetActive(false);
      Registry.DocumentService.HasOpenPanels = false;

      var walker = CreateWalker();
      var refRegistry = new RefRegistry();
      var root = walker.Walk(refRegistry);

      var questRegion = root.Children.FirstOrDefault(c => c.Label == "Quest");
      Assert.IsNotNull(questRegion, "Should have a Quest region");

      var scene3dRegion = root.Children.FirstOrDefault(c => c.Label == "Scene3D");
      Assert.IsNull(scene3dRegion, "Should not have a Scene3D region in quest mode");
    }

    // -- Test 4: Occlusion --

    /// <summary>
    /// Set HasOpenPanels = true. Walk in non-battle mode. Verify quest content
    /// groups are omitted but Controls and UIToolkit still appear under Quest.
    /// </summary>
    [UnityTest]
    public IEnumerator Occlusion_QuestContentOmittedWhenPanelsOpen()
    {
      yield return Initialize();
      SetUpUIDocument();

      Registry.BattleLayout.Contents.SetActive(false);
      Registry.DocumentService.HasOpenPanels = true;

      // Add a UI Toolkit element that should still appear
      var uiElement = new NodeVisualElement { name = "StillVisible" };
      uiElement.pickingMode = PickingMode.Position;
      Registry.DocumentService.RootVisualElement.Add(uiElement);

      var walker = CreateWalker();
      var refRegistry = new RefRegistry();
      var root = walker.Walk(refRegistry);

      var questRegion = root.Children.FirstOrDefault(c => c.Label == "Quest");
      Assert.IsNotNull(questRegion, "Should have a Quest region");

      // Controls should still be present
      var controlsGroup = FindNode(questRegion!, n => n.Label == "Controls");
      Assert.IsNotNull(controlsGroup, "Controls should still be present when panels are open");

      // Map, Essence, etc. should NOT be present when panels are open
      var mapGroup = FindNode(questRegion!, n => n.Label == "Map");
      Assert.IsNull(mapGroup, "Map should be omitted when HasOpenPanels is true");

      // UI Toolkit element should still be present under Quest
      var uiToolkitRegion = FindNode(questRegion!, n => n.Label == "UIToolkit");
      Assert.IsNotNull(uiToolkitRegion, "UIToolkit should still be present when panels are open");
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

      Registry.BattleLayout.Contents.SetActive(false);

      var element = new NodeVisualElement { name = "ClickTarget" };
      element.pickingMode = PickingMode.Position;
      var callbacks = element.Callbacks.Value;
      var clickFired = false;
      callbacks.SetCallback(
        element,
        Callbacks.Event.Click,
        () =>
        {
          clickFired = true;
        }
      );

      Registry.DocumentService.RootVisualElement.Add(element);

      var walker = CreateWalker();
      var refRegistry = new RefRegistry();
      var root = walker.Walk(refRegistry);

      // Find the interactive element and its ref
      var targetNode = FindNode(root, n => n.Label == "ClickTarget" && n.Interactive);
      Assert.IsNotNull(targetNode, "Should find the clickable element");

      Assert.IsTrue(
        refRegistry.TryGetCallbacks("e1", out var refCallbacks),
        "Should have at least one ref registered"
      );

      refCallbacks.OnClick?.Invoke();

      Assert.IsTrue(clickFired, "Click callback should have been invoked via ref dispatch");
    }

    // -- Test 6: Tree structure (non-battle) --

    /// <summary>
    /// Verify the non-battle snapshot tree has the expected structure with
    /// application root and a Quest region child.
    /// </summary>
    [UnityTest]
    public IEnumerator TreeStructure_NonBattleHasApplicationRootWithQuestRegion()
    {
      yield return Initialize();
      SetUpUIDocument();

      Registry.BattleLayout.Contents.SetActive(false);

      var walker = CreateWalker();
      var refRegistry = new RefRegistry();
      var root = walker.Walk(refRegistry);

      Assert.AreEqual("application", root.Role);
      Assert.AreEqual("Dreamtides", root.Label);
      Assert.AreEqual(1, root.Children.Count, "Should have a Quest region");
      Assert.AreEqual("region", root.Children[0].Role);
      Assert.AreEqual("Quest", root.Children[0].Label);
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

      Registry.BattleLayout.Contents.SetActive(false);

      var element = new NodeVisualElement { name = "HoverTarget" };
      element.pickingMode = PickingMode.Position;
      var callbacks = element.Callbacks.Value;
      var hoverFired = false;
      callbacks.SetCallback(
        element,
        Callbacks.Event.MouseEnter,
        () =>
        {
          hoverFired = true;
        }
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

    // -- Test 8: Battle mode tree structure --

    /// <summary>
    /// Verify that in battle mode, the tree has a "Battle" region with
    /// "Controls", "User", and "Opponent" groups.
    /// </summary>
    [UnityTest]
    public IEnumerator BattleMode_HasBattleRegionWithPlayerGroups()
    {
      yield return Initialize();
      SetUpUIDocument();

      Registry.BattleLayout.Contents.SetActive(true);
      Registry.DocumentService.HasOpenPanels = false;

      var walker = CreateWalker();
      var refRegistry = new RefRegistry();
      var root = walker.Walk(refRegistry);

      Assert.AreEqual("application", root.Role);
      Assert.AreEqual("Dreamtides", root.Label);

      var battleRegion = root.Children.FirstOrDefault(c => c.Label == "Battle");
      Assert.IsNotNull(battleRegion, "Should have a Battle region");
      Assert.AreEqual("region", battleRegion!.Role);

      var controlsGroup = battleRegion.Children.FirstOrDefault(c => c.Label == "Controls");
      Assert.IsNotNull(controlsGroup, "Battle should have a Controls group");

      var userGroup = battleRegion.Children.FirstOrDefault(c => c.Label == "User");
      Assert.IsNotNull(userGroup, "Battle should have a User group");

      var opponentGroup = battleRegion.Children.FirstOrDefault(c => c.Label == "Opponent");
      Assert.IsNotNull(opponentGroup, "Battle should have an Opponent group");
    }

    // -- Test 9: Battle mode user status --

    /// <summary>
    /// Verify that the user's status subgroup includes energy, score, spark,
    /// and turn labels.
    /// </summary>
    [UnityTest]
    public IEnumerator BattleMode_UserStatusHasLabels()
    {
      yield return Initialize();
      SetUpUIDocument();

      Registry.BattleLayout.Contents.SetActive(true);
      Registry.DocumentService.HasOpenPanels = false;

      // Set status values
      var userStatus = Registry.BattleLayout.UserStatusDisplay;
      userStatus.SetEnergy(3, 7, false);
      userStatus.SetScore(2, false);
      userStatus.SetTotalSpark(49, false);

      // Create turn indicator GameObjects (not created by generated layout)
      var leftIndicator = new GameObject("LeftTurnIndicator");
      leftIndicator.transform.SetParent(userStatus.transform);
      userStatus._leftTurnIndicator = leftIndicator;
      var rightIndicator = new GameObject("RightTurnIndicator");
      rightIndicator.transform.SetParent(userStatus.transform);
      userStatus._rightTurnIndicator = rightIndicator;
      leftIndicator.SetActive(true);

      var walker = CreateWalker();
      var refRegistry = new RefRegistry();
      var root = walker.Walk(refRegistry);

      var battleRegion = root.Children.FirstOrDefault(c => c.Label == "Battle");
      Assert.IsNotNull(battleRegion);
      var userGroup = FindNode(battleRegion!, n => n.Label == "User");
      Assert.IsNotNull(userGroup);
      var statusGroup = FindNode(userGroup!, n => n.Label == "Status");
      Assert.IsNotNull(statusGroup, "User should have a Status group");

      var energyLabel = FindNode(
        statusGroup!,
        n => n.Label != null && n.Label.StartsWith("Energy:")
      );
      Assert.IsNotNull(energyLabel, "Status should have an Energy label");

      var scoreLabel = FindNode(statusGroup!, n => n.Label != null && n.Label.StartsWith("Score:"));
      Assert.IsNotNull(scoreLabel, "Status should have a Score label");

      var sparkLabel = FindNode(statusGroup!, n => n.Label != null && n.Label.StartsWith("Spark:"));
      Assert.IsNotNull(sparkLabel, "Status should have a Spark label");

      var turnLabel = FindNode(statusGroup!, n => n.Label != null && n.Label.StartsWith("Turn:"));
      Assert.IsNotNull(turnLabel, "Status should have a Turn label");
    }

    // -- Test 10: Battle mode card on battlefield --

    /// <summary>
    /// Place a revealed card on the user battlefield. Walk. Verify it appears
    /// under the User > Battlefield group with spark annotation.
    /// </summary>
    [UnityTest]
    public IEnumerator BattleMode_RevealedCardAppearsonBattlefield()
    {
      yield return Initialize();
      SetUpUIDocument();

      Registry.BattleLayout.Contents.SetActive(true);
      Registry.DocumentService.HasOpenPanels = false;

      // Initialize the battlefield layout's GameContext before adding cards
      Registry.BattleLayout.UserBattlefield._internalGameContext = GameContext.Battlefield;

      var card = CreateTestCard();
      card._cardView.Revealed = new RevealedCardView
      {
        Name = "Test Knight",
        CardType = "Character",
        Spark = "5",
      };
      card.GameContext = GameContext.Battlefield;
      Registry.BattleLayout.UserBattlefield.Add(card);

      var walker = CreateWalker();
      var refRegistry = new RefRegistry();
      var root = walker.Walk(refRegistry);

      var battleRegion = root.Children.FirstOrDefault(c => c.Label == "Battle");
      Assert.IsNotNull(battleRegion);
      var userGroup = FindNode(battleRegion!, n => n.Label == "User");
      Assert.IsNotNull(userGroup);
      var battlefieldGroup = FindNode(userGroup!, n => n.Label == "Battlefield");
      Assert.IsNotNull(battlefieldGroup, "User should have a Battlefield group");

      var cardNode = FindNode(
        battlefieldGroup!,
        n => n.Label != null && n.Label.Contains("Test Knight") && n.Label.Contains("spark: 5")
      );
      Assert.IsNotNull(cardNode, "Card should appear on battlefield with spark annotation");
      Assert.IsTrue(cardNode!.Interactive);
    }

    // -- Test 11: Battle mode thinking indicator --

    /// <summary>
    /// Enable the thinking indicator. Walk. Verify the label appears.
    /// </summary>
    [UnityTest]
    public IEnumerator BattleMode_ThinkingIndicatorShown()
    {
      yield return Initialize();
      SetUpUIDocument();

      Registry.BattleLayout.Contents.SetActive(true);
      Registry.DocumentService.HasOpenPanels = false;
      Registry.BattleLayout.ThinkingIndicator.SetActive(true);

      var walker = CreateWalker();
      var refRegistry = new RefRegistry();
      var root = walker.Walk(refRegistry);

      var battleRegion = root.Children.FirstOrDefault(c => c.Label == "Battle");
      Assert.IsNotNull(battleRegion);

      var thinkingLabel = FindNode(battleRegion!, n => n.Label == "Opponent is thinking...");
      Assert.IsNotNull(thinkingLabel, "Thinking indicator should appear as a label");
    }

    // -- Test 12: Battle mode panels open hides 3D content --

    /// <summary>
    /// In battle mode with HasOpenPanels = true, verify that User/Opponent
    /// groups are not present but Controls still are.
    /// </summary>
    [UnityTest]
    public IEnumerator BattleMode_PanelsOpenHides3DContent()
    {
      yield return Initialize();
      SetUpUIDocument();

      Registry.BattleLayout.Contents.SetActive(true);
      Registry.DocumentService.HasOpenPanels = true;

      var walker = CreateWalker();
      var refRegistry = new RefRegistry();
      var root = walker.Walk(refRegistry);

      var battleRegion = root.Children.FirstOrDefault(c => c.Label == "Battle");
      Assert.IsNotNull(battleRegion);

      var controlsGroup = FindNode(battleRegion!, n => n.Label == "Controls");
      Assert.IsNotNull(controlsGroup, "Controls should still be present when panels are open");

      var userGroup = FindNode(battleRegion!, n => n.Label == "User");
      Assert.IsNull(userGroup, "User group should not be present when panels are open");

      var opponentGroup = FindNode(battleRegion!, n => n.Label == "Opponent");
      Assert.IsNull(opponentGroup, "Opponent group should not be present when panels are open");
    }

    // -- Test 13: Rich text stripping --

    /// <summary>
    /// Verify that card names with rich text tags and newlines are properly
    /// cleaned up in labels.
    /// </summary>
    [UnityTest]
    public IEnumerator BattleMode_RichTextStrippedFromCardLabels()
    {
      yield return Initialize();
      SetUpUIDocument();

      Registry.BattleLayout.Contents.SetActive(true);
      Registry.DocumentService.HasOpenPanels = false;

      // Initialize the battlefield layout's GameContext before adding cards
      Registry.BattleLayout.UserBattlefield._internalGameContext = GameContext.Battlefield;

      var card = CreateTestCard();
      card._cardView.Revealed = new RevealedCardView
      {
        Name = "The Black Knight\n<size=75%>Malignant Usurper</size>",
        CardType = "Character",
        Spark = "5",
      };
      card.GameContext = GameContext.Battlefield;
      Registry.BattleLayout.UserBattlefield.Add(card);

      var walker = CreateWalker();
      var refRegistry = new RefRegistry();
      var root = walker.Walk(refRegistry);

      var cardNode = FindNode(
        root,
        n => n.Label != null && n.Label.Contains("The Black Knight, Malignant Usurper")
      );
      Assert.IsNotNull(cardNode, "Card name should have rich text stripped and newline replaced");
      Assert.IsFalse(cardNode!.Label!.Contains("<size"), "Label should not contain rich text tags");
    }

    // -- Test 14: Icon characters stripped from status labels --

    /// <summary>
    /// Set energy/score values, walk, assert exact label text without icon
    /// artifacts.
    /// </summary>
    [UnityTest]
    public IEnumerator BattleMode_IconCharactersStrippedFromStatusLabels()
    {
      yield return Initialize();
      SetUpUIDocument();

      Registry.BattleLayout.Contents.SetActive(true);
      Registry.DocumentService.HasOpenPanels = false;

      var userStatus = Registry.BattleLayout.UserStatusDisplay;
      userStatus.SetEnergy(3, 7, false);
      userStatus.SetScore(2, false);

      var walker = CreateWalker();
      var refRegistry = new RefRegistry();
      var root = walker.Walk(refRegistry);

      var battleRegion = root.Children.FirstOrDefault(c => c.Label == "Battle");
      Assert.IsNotNull(battleRegion);
      var userGroup = FindNode(battleRegion!, n => n.Label == "User");
      Assert.IsNotNull(userGroup);
      var statusGroup = FindNode(userGroup!, n => n.Label == "Status");
      Assert.IsNotNull(statusGroup);

      var energyLabel = FindNode(
        statusGroup!,
        n => n.Label != null && n.Label.StartsWith("Energy:")
      );
      Assert.IsNotNull(energyLabel);
      Assert.AreEqual("Energy: 3/7", energyLabel!.Label);

      var scoreLabel = FindNode(statusGroup!, n => n.Label != null && n.Label.StartsWith("Score:"));
      Assert.IsNotNull(scoreLabel);
      Assert.AreEqual("Score: 2", scoreLabel!.Label);
    }

    // -- Test 15: Card labels exclude interaction annotations --

    /// <summary>
    /// Create a hand card with CanPlay/OnClick actions, walk, assert label
    /// contains cost but NOT drag/click annotations.
    /// </summary>
    [UnityTest]
    public IEnumerator BattleMode_CardLabelsExcludeInteractionAnnotations()
    {
      yield return Initialize();
      SetUpUIDocument();

      Registry.BattleLayout.Contents.SetActive(true);
      Registry.DocumentService.HasOpenPanels = false;

      Registry.BattleLayout.UserHand._layout1._internalGameContext = GameContext.Hand;

      var card = CreateTestCard();
      card._cardView.Revealed = new RevealedCardView
      {
        Name = "Minstrel of Falling Light\n<size=75%>Musician</size>",
        CardType = "Character",
        Cost = "2",
        Actions = new CardActions { CanPlay = GameActionEnum.NoOp, OnClick = GameActionEnum.NoOp },
      };
      card.GameContext = GameContext.Hand;
      Registry.BattleLayout.UserHand.Add(card);

      var walker = CreateWalker();
      var refRegistry = new RefRegistry();
      var root = walker.Walk(refRegistry);

      var battleRegion = root.Children.FirstOrDefault(c => c.Label == "Battle");
      Assert.IsNotNull(battleRegion);
      var userGroup = FindNode(battleRegion!, n => n.Label == "User");
      Assert.IsNotNull(userGroup);

      var cardNode = FindNode(
        userGroup!,
        n => n.Label != null && n.Label.Contains("Minstrel of Falling Light")
      );
      Assert.IsNotNull(cardNode, "Card should appear in hand");
      Assert.IsTrue(cardNode!.Label!.Contains("cost: 2"), "Label should contain cost annotation");
      Assert.IsFalse(
        cardNode.Label.Contains("drag to play"),
        "Label should NOT contain drag to play"
      );
      Assert.IsFalse(
        cardNode.Label.Contains("click to select"),
        "Label should NOT contain click to select"
      );
    }

    // -- Test 16: Empty UIToolkit containers filtered --

    /// <summary>
    /// Walk battle mode with empty overlay containers. Assert no UIToolkit
    /// region when overlay containers are empty.
    /// </summary>
    [UnityTest]
    public IEnumerator BattleMode_EmptyUiToolkitContainersFiltered()
    {
      yield return Initialize();
      SetUpUIDocument();

      Registry.BattleLayout.Contents.SetActive(true);
      Registry.DocumentService.HasOpenPanels = false;

      // Add an unlabeled interactive element (simulates empty overlay container)
      var element = new NodeVisualElement();
      element.pickingMode = PickingMode.Position;
      Registry.DocumentService.RootVisualElement.Add(element);

      var walker = CreateWalker();
      var refRegistry = new RefRegistry();
      var root = walker.Walk(refRegistry);

      var battleRegion = root.Children.FirstOrDefault(c => c.Label == "Battle");
      Assert.IsNotNull(battleRegion);

      var uiToolkitRegion = FindNode(battleRegion!, n => n.Label == "UIToolkit");
      Assert.IsNull(
        uiToolkitRegion,
        "UIToolkit region should not appear when overlay containers are empty"
      );
    }

    // -- Test 17: UIToolkit containers with real content shown --

    /// <summary>
    /// Add a named interactive element, walk, assert UIToolkit region appears
    /// with the element.
    /// </summary>
    [UnityTest]
    public IEnumerator BattleMode_UiToolkitContainersWithRealContentShown()
    {
      yield return Initialize();
      SetUpUIDocument();

      Registry.BattleLayout.Contents.SetActive(true);
      Registry.DocumentService.HasOpenPanels = false;

      var element = new NodeVisualElement { name = "RealOverlayButton" };
      element.pickingMode = PickingMode.Position;
      Registry.DocumentService.RootVisualElement.Add(element);

      var walker = CreateWalker();
      var refRegistry = new RefRegistry();
      var root = walker.Walk(refRegistry);

      var battleRegion = root.Children.FirstOrDefault(c => c.Label == "Battle");
      Assert.IsNotNull(battleRegion);

      var uiToolkitRegion = FindNode(battleRegion!, n => n.Label == "UIToolkit");
      Assert.IsNotNull(uiToolkitRegion, "UIToolkit region should appear with real content");

      var buttonNode = FindNode(
        uiToolkitRegion!,
        n => n.Label == "RealOverlayButton" && n.Interactive
      );
      Assert.IsNotNull(buttonNode, "Named interactive element should be in the tree");
    }

    // -- Test 18: Essence label appears during battle --

    /// <summary>
    /// Set essence value via _originalText, walk, assert "Essence: 42" label
    /// exists.
    /// </summary>
    [UnityTest]
    public IEnumerator BattleMode_EssenceLabelAppearsDuringBattle()
    {
      yield return Initialize();
      SetUpUIDocument();

      Registry.BattleLayout.Contents.SetActive(true);
      Registry.DocumentService.HasOpenPanels = false;

      Registry.DreamscapeLayout.EssenceTotal._originalText = "42";

      var walker = CreateWalker();
      var refRegistry = new RefRegistry();
      var root = walker.Walk(refRegistry);

      var battleRegion = root.Children.FirstOrDefault(c => c.Label == "Battle");
      Assert.IsNotNull(battleRegion);

      var essenceLabel = FindNode(battleRegion!, n => n.Label == "Essence: 42");
      Assert.IsNotNull(essenceLabel, "Essence label should appear during battle");
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
