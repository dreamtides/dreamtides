#nullable enable

using System;
using System.Collections.Generic;
using Abu;
using Dreamtides.Buttons;
using Dreamtides.Components;
using Dreamtides.Layout;
using Dreamtides.Schema;
using UnityEngine;
using Object = UnityEngine.Object;

namespace Dreamtides.Abu
{
  public partial class DreamtidesSceneWalker
  {
    // ── Battle mode walk ──────────────────────────────────────────────

    AbuSceneNode WalkBattle(RefRegistry refRegistry)
    {
      var region = CreateRegionNode("Battle");
      var layout = _registry.BattleLayout;
      var hasOpenPanels = _registry.DocumentService.HasOpenPanels;

      // Build browser button lookup
      var browserButtons = new Dictionary<CardBrowserType, CardBrowserButton>();
      foreach (
        var btn in Object.FindObjectsByType<CardBrowserButton>(
          FindObjectsInactive.Exclude,
          FindObjectsSortMode.None
        )
      )
      {
        browserButtons[btn._type] = btn;
      }

      // 1. Controls (always shown)
      region.Children.Add(WalkControls(refRegistry));

      if (!hasOpenPanels)
      {
        // 2. User
        region.Children.Add(
          WalkPlayer(
            "User",
            layout.UserStatusDisplay,
            layout.UserBattlefield,
            layout.UserHand.Objects,
            browserButtons,
            CardBrowserType.UserDeck,
            CardBrowserType.UserVoid,
            CardBrowserType.UserStatus,
            isUser: true,
            refRegistry
          )
        );

        // 3. Opponent
        region.Children.Add(
          WalkPlayer(
            "Opponent",
            layout.EnemyStatusDisplay,
            layout.EnemyBattlefield,
            layout.EnemyHand.Objects,
            browserButtons,
            CardBrowserType.EnemyDeck,
            CardBrowserType.EnemyVoid,
            CardBrowserType.EnemyStatus,
            isUser: false,
            refRegistry
          )
        );

        // 4. Stack (if any cards on stack)
        var stackGroup = WalkStack(layout, refRegistry);
        if (stackGroup != null)
        {
          region.Children.Add(stackGroup);
        }

        // 5. Game modifiers (if any)
        var modifiersGroup = WalkObjectLayoutGroup(
          "Game Modifiers",
          layout.GameModifiersDisplay,
          refRegistry
        );
        if (modifiersGroup != null)
        {
          region.Children.Add(modifiersGroup);
        }

        // 6. Action buttons
        var actionsGroup = WalkActionButtons(layout, refRegistry);
        if (actionsGroup.Children.Count > 0)
        {
          region.Children.Add(actionsGroup);
        }

        // 7. Playable cards summary
        AddPlayableCardsSummary(region, layout.UserHand.Objects);

        // 8. Essence label
        AddEssenceLabel(region);

        // 9. Play zone (drag target for playing cards from hand)
        AddInteractiveNode(region, refRegistry, "target", "Play Zone", new RefCallbacks());

        // 10. Thinking indicator
        if (layout.ThinkingIndicator.activeSelf)
        {
          region.Children.Add(CreateLabelNode("Opponent is thinking..."));
        }
      }

      // Card Order Selector (shown when active)
      var cardOrderGroup = WalkCardOrderSelector(layout, refRegistry);
      if (cardOrderGroup != null)
      {
        region.Children.Add(cardOrderGroup);
      }

      // Browser (e.g. modal effect choices, shown when active)
      var browserGroup = WalkObjectLayoutGroup("Browser", layout.Browser, refRegistry);
      if (browserGroup != null)
      {
        region.Children.Add(browserGroup);
      }

      // 11. UI overlays (filtered, only when content exists)
      var uiOverlay = WalkUiToolkitFiltered(refRegistry);
      if (uiOverlay != null)
      {
        region.Children.Add(uiOverlay);
      }

      return region;
    }

    void AddPlayableCardsSummary(AbuSceneNode parent, IReadOnlyList<Displayable> handObjects)
    {
      if (!_registry.CapabilitiesService.CanPlayCards())
      {
        return;
      }

      var playableCards = new List<string>();
      foreach (var obj in handObjects)
      {
        if (
          obj is not Card card
          || !card.CanHandleMouseEvents()
          || card.CardView.Revealed is not { } revealed
          || revealed.Actions.CanPlay is not { } canPlayAction
          || canPlayAction.IsNull
        )
        {
          continue;
        }

        var name = ToSingleLineText(revealed.Name, fallback: "Unknown");
        var cost = StripRichText(revealed.Cost);
        playableCards.Add(!string.IsNullOrEmpty(cost) ? $"{name} (cost: {cost})" : name);
      }

      if (playableCards.Count == 0)
      {
        return;
      }

      parent.Children.Add(CreateLabelNode($"Playable Cards: {string.Join(", ", playableCards)}"));
    }

    // ── Controls ──────────────────────────────────────────────────────

    AbuSceneNode WalkControls(RefRegistry refRegistry)
    {
      var group = CreateGroupNode("Controls");
      var doc = _registry.DocumentService;

      TryAddCanvasButton(group, refRegistry, doc.MenuButton, "Menu");
      TryAddCanvasButton(group, refRegistry, doc.UndoButton, "Undo");
      TryAddCanvasButton(group, refRegistry, doc.DevButton, "Dev");
      TryAddCanvasButton(group, refRegistry, doc.BugButton, "Bug Report");
      TryAddCloseButton(group, refRegistry, _registry.BattleLayout.CloseBrowserButton, "Close Browser");

      return group;
    }

    // ── Player ────────────────────────────────────────────────────────

    AbuSceneNode WalkPlayer(
      string playerLabel,
      PlayerStatusDisplay statusDisplay,
      ObjectLayout battlefield,
      IReadOnlyList<Displayable> handObjects,
      Dictionary<CardBrowserType, CardBrowserButton> browserButtons,
      CardBrowserType deckType,
      CardBrowserType voidType,
      CardBrowserType statusType,
      bool isUser,
      RefRegistry refRegistry
    )
    {
      var group = CreateGroupNode(playerLabel);

      // Status
      group.Children.Add(WalkStatus(statusDisplay, isUser, handObjects, battlefield));

      // Deck browser
      AddBrowserButton(group, browserButtons, deckType, "Deck", refRegistry);

      // Identity browser
      AddBrowserButton(group, browserButtons, statusType, "Identity", refRegistry);

      // Void browser
      AddBrowserButton(group, browserButtons, voidType, "Void", refRegistry);

      // Battlefield
      var battlefieldGroup = WalkCardsGroup("Battlefield", "Battlefield", battlefield.Objects, refRegistry);
      if (battlefieldGroup != null)
      {
        group.Children.Add(battlefieldGroup);
      }

      var handCounts = CountHandCards(handObjects);

      // Hand
      if (isUser)
      {
        var handGroup = WalkCardsGroup(BuildUserHandLabel(handCounts), "Hand", handObjects, refRegistry);
        if (handGroup != null)
        {
          group.Children.Add(handGroup);
        }
      }
      else if (handObjects.Count > 0)
      {
        group.Children.Add(CreateLabelNode(BuildOpponentHandSummaryLabel(handCounts)));
      }

      return group;
    }

    AbuSceneNode? WalkCardsGroup(
      string label,
      string zoneContext,
      IEnumerable<Displayable> objects,
      RefRegistry refRegistry
    )
    {
      var group = CreateGroupNode(label);
      AddCardNodes(group, objects, zoneContext, refRegistry);

      return group.Children.Count > 0 ? group : null;
    }

    void AddCardNodes(
      AbuSceneNode parent,
      IEnumerable<Displayable> objects,
      string zoneContext,
      RefRegistry refRegistry
    )
    {
      foreach (var obj in objects)
      {
        var cardNode = BuildCardNode(obj, zoneContext, refRegistry);
        if (cardNode != null)
        {
          parent.Children.Add(cardNode);
        }
      }
    }

    static string BuildUserHandLabel(HandCounts counts)
    {
      return counts.AbilityCount > 0
        ? $"Hand ({counts.CardCount} cards, {counts.AbilityCount} abilities)"
        : $"Hand ({counts.CardCount} cards)";
    }

    static string BuildOpponentHandSummaryLabel(HandCounts counts)
    {
      return counts.AbilityCount > 0
        ? $"Hand: {counts.CardCount} cards, {counts.AbilityCount} abilities"
        : $"Hand: {counts.CardCount} cards";
    }

    // ── Status ────────────────────────────────────────────────────────

    AbuSceneNode WalkStatus(
      PlayerStatusDisplay statusDisplay,
      bool isUser,
      IReadOnlyList<Displayable> handObjects,
      ObjectLayout battlefield
    )
    {
      var group = CreateGroupNode("Status");

      var energyText = StripRichText(statusDisplay._energy._originalText);
      if (!string.IsNullOrEmpty(energyText))
      {
        group.Children.Add(CreateLabelNode($"Energy: {energyText}"));
      }

      var scoreText = StripRichText(statusDisplay._score._originalText);
      if (!string.IsNullOrEmpty(scoreText))
      {
        group.Children.Add(CreateLabelNode($"Score: {scoreText}"));
      }

      var sparkText = StripRichText(statusDisplay._totalSpark._originalText);
      if (!string.IsNullOrEmpty(sparkText))
      {
        var sparkLabel = $"Spark: {sparkText}";
        var characterSpark = 0;
        var characterCount = 0;
        foreach (var obj in battlefield.Objects)
        {
          if (obj is Card card && card.CardView.Revealed is { } rev)
          {
            var cardSparkText = StripRichText(rev.Spark);
            if (int.TryParse(cardSparkText, out var cardSpark))
            {
              characterSpark += cardSpark;
              characterCount++;
            }
          }
        }

        if (characterCount > 0 && int.TryParse(sparkText, out var totalSpark))
        {
          var bonus = totalSpark - characterSpark;
          sparkLabel = bonus != 0
            ? $"Spark: {sparkText} ({characterSpark} from {characterCount} characters + {bonus} bonus)"
            : $"Spark: {sparkText} (from {characterCount} characters)";
        }

        group.Children.Add(CreateLabelNode(sparkLabel));
      }

      if (isUser)
      {
        var hasTurn =
          (statusDisplay._leftTurnIndicator != null && statusDisplay._leftTurnIndicator.activeSelf)
          || (
            statusDisplay._rightTurnIndicator != null
            && statusDisplay._rightTurnIndicator.activeSelf
          );
        var turnNumber = (_registry.BattleLayout.TurnNumber / 2) + 1;
        var turnOwner = hasTurn ? "yours" : "opponent's";
        group.Children.Add(CreateLabelNode($"Turn: {turnNumber} ({turnOwner})"));
      }
      else
      {
        group.Children.Add(CreateLabelNode($"Hand: {CountHandCards(handObjects).CardCount} cards"));
      }

      return group;
    }

    // ── Browser buttons ───────────────────────────────────────────────

    void AddBrowserButton(
      AbuSceneNode parent,
      Dictionary<CardBrowserType, CardBrowserButton> browserButtons,
      CardBrowserType type,
      string zoneName,
      RefRegistry refRegistry
    )
    {
      if (!browserButtons.TryGetValue(type, out var button))
      {
        return;
      }

      var count = GetBrowserZoneCount(type);

      if (type is CardBrowserType.UserStatus or CardBrowserType.EnemyStatus)
      {
        // Identity always shows as a button
        AddInteractiveNode(
          parent,
          refRegistry,
          "button",
          $"Browse {zoneName}",
          BuildDisplayableCallbacks(button)
        );
        return;
      }

      if (count > 0)
      {
        AddInteractiveNode(
          parent,
          refRegistry,
          "button",
          $"Browse {zoneName} ({count} cards)",
          BuildDisplayableCallbacks(button)
        );
        return;
      }

      parent.Children.Add(CreateLabelNode($"{zoneName}: 0 cards"));
    }

    int GetBrowserZoneCount(CardBrowserType type)
    {
      var layout = _registry.BattleLayout;
      return type switch
      {
        CardBrowserType.UserDeck => layout.UserDeck.Objects.Count,
        CardBrowserType.EnemyDeck => layout.EnemyDeck.Objects.Count,
        CardBrowserType.UserVoid => layout.UserVoid.Objects.Count,
        CardBrowserType.EnemyVoid => layout.EnemyVoid.Objects.Count,
        _ => 0,
      };
    }

    // ── Card nodes ────────────────────────────────────────────────────

    AbuSceneNode? BuildCardNode(
      Displayable displayable,
      string zoneContext,
      RefRegistry refRegistry
    )
    {
      if (displayable is not Card card)
      {
        return null;
      }

      if (!card.CanHandleMouseEvents())
      {
        return null;
      }

      // Token cards on the stack represent triggered abilities and should
      // not appear in the snapshot tree.
      if (zoneContext == "Stack" && card.CardView.Prefab == CardPrefab.Token)
      {
        return null;
      }

      var revealed = card.CardView.Revealed;
      if (revealed == null)
      {
        return null;
      }

      var canPlay =
        zoneContext == "Hand"
        && revealed.Actions.CanPlay is { } cp
        && !cp.IsNull
        && _registry.CapabilitiesService.CanPlayCards();

      var label = BuildCardLabel(revealed, zoneContext, canPlay);

      if (zoneContext == "Stack" && revealed.InfoZoomData?.Icons is { Count: > 0 } icons)
      {
        var targetNames = new List<string>();
        foreach (var icon in icons)
        {
          var targetCard = _registry.CardService.GetCardIfExists(icon.CardId);
          var targetName = ToSingleLineText(targetCard?.CardView.Revealed?.Name);
          if (!string.IsNullOrEmpty(targetName))
          {
            targetNames.Add(targetName);
          }
        }

        if (targetNames.Count > 0)
        {
          label += $" (targeting: {string.Join(", ", targetNames)})";
        }
      }

      var node = CreateButtonNode(label);
      var callbacks = BuildDisplayableCallbacks(card);
      if (canPlay)
      {
        callbacks.OnDrag = _ =>
        {
          var action = card.CardView.Revealed?.Actions?.CanPlay?.ToGameAction();
          if (action.HasValue)
          {
            _registry.ActionService.PerformAction(action.Value);
          }
        };
      }
      refRegistry.Register(callbacks);
      return node;
    }

    static string BuildCardLabel(RevealedCardView revealed, string zoneContext, bool canPlay)
    {
      var name = ToSingleLineText(revealed.Name, fallback: "Unknown");
      var cardType = StripRichText(revealed.CardType);
      var showDetails = zoneContext is "Hand" or "Battlefield" or "Stack" or "Browser";

      var annotations = new List<string>();

      if (zoneContext is "Hand" or "Browser" && !string.IsNullOrEmpty(revealed.Cost))
      {
        annotations.Add($"cost: {StripRichText(revealed.Cost)}");
      }

      if (showDetails && !string.IsNullOrEmpty(revealed.Spark))
      {
        annotations.Add($"spark: {StripRichText(revealed.Spark)}");
      }

      if (canPlay)
      {
        annotations.Add("can play");
      }

      if (IsSelectionColor(revealed.OutlineColor))
      {
        annotations.Add("selected");
      }

      var suffix = annotations.Count > 0 ? $" ({string.Join(", ", annotations)})" : "";
      string label;
      if (!string.IsNullOrEmpty(cardType))
      {
        label = $"{name}, {cardType}{suffix}";
      }
      else
      {
        label = $"{name}{suffix}";
      }

      if (showDetails)
      {
        var rulesText = StripRichText(revealed.RulesText)?.Replace("\n", " ");
        if (!string.IsNullOrEmpty(rulesText))
        {
          label += $" -- {rulesText}";
        }
      }

      return label;
    }

    /// <summary>
    /// Returns true if the given color matches the YELLOW_500 selection color.
    /// </summary>
    static bool IsSelectionColor(DisplayColor? color)
    {
      if (color == null)
      {
        return false;
      }

      const double epsilon = 0.01;
      return Math.Abs(color.Red - 1.0) < epsilon
        && Math.Abs(color.Green - 0.92) < epsilon
        && Math.Abs(color.Blue - 0.23) < epsilon;
    }

    // ── Action buttons ────────────────────────────────────────────────

    AbuSceneNode WalkActionButtons(BattleLayout layout, RefRegistry refRegistry)
    {
      var group = CreateGroupNode("Actions");
      TryAddActionButton(group, layout.PrimaryActionButton, refRegistry);
      TryAddActionButton(group, layout.SecondaryActionButton, refRegistry);
      TryAddActionButton(group, layout.IncrementActionButton, refRegistry);
      TryAddActionButton(group, layout.DecrementActionButton, refRegistry);
      return group;
    }

    void TryAddActionButton(AbuSceneNode parent, ActionButton button, RefRegistry refRegistry)
    {
      if (!button.gameObject.activeSelf || !button._text.gameObject.activeSelf)
      {
        return;
      }

      var label = StripRichText(button._text.text);
      if (string.IsNullOrEmpty(label))
      {
        return;
      }

      AddInteractiveNode(parent, refRegistry, "button", label, BuildDisplayableCallbacks(button));
    }

    // ── Essence ───────────────────────────────────────────────────────

    void AddEssenceLabel(AbuSceneNode parent)
    {
      var essenceComponent = _registry.DreamscapeLayout.EssenceTotal;
      var essenceText = essenceComponent._originalText;
      if (string.IsNullOrEmpty(essenceText))
      {
        essenceText = StripRichText(essenceComponent._text.text);
      }

      if (!string.IsNullOrEmpty(essenceText))
      {
        parent.Children.Add(CreateLabelNode($"Essence: {essenceText}"));
      }
    }

    // ── Stack ─────────────────────────────────────────────────────────

    AbuSceneNode? WalkStack(BattleLayout layout, RefRegistry refRegistry)
    {
      var stackGroup = CreateGroupNode("Stack");

      AddStackObjects(stackGroup, layout.DefaultStack, refRegistry);
      AddStackObjects(stackGroup, layout.TargetingUserStack, refRegistry);
      AddStackObjects(stackGroup, layout.TargetingEnemyStack, refRegistry);
      AddStackObjects(stackGroup, layout.TargetingBothStack, refRegistry);

      var count = stackGroup.Children.Count;
      if (count > 1)
      {
        for (var i = 0; i < count; i++)
        {
          var child = stackGroup.Children[i];
          var position = i == count - 1 ? $"{i + 1} of {count}, top" : $"{i + 1} of {count}";
          child.Label = $"{child.Label} [{position}]";
        }
      }

      return count > 0 ? stackGroup : null;
    }

    void AddStackObjects(AbuSceneNode parent, ObjectLayout stack, RefRegistry refRegistry)
    {
      AddCardNodes(parent, stack.Objects, "Stack", refRegistry);
    }

    // ── Card Order Selector ────────────────────────────────────────────

    AbuSceneNode? WalkCardOrderSelector(BattleLayout layout, RefRegistry refRegistry)
    {
      var selector = layout.CardOrderSelector;
      if (!selector.IsOpen)
      {
        return null;
      }

      var group = CreateGroupNode("Card Order Selector");

      // Build target ref lookup so card OnDrag can map target refs to actions
      var targetRefToPosition = new Dictionary<string, CardOrderSelectionTarget>();

      // Deck position targets: 0..N for N cards currently in deck
      var deckCount = selector.Objects.Count;
      for (var i = 0; i <= deckCount; i++)
      {
        var position = i;
        var targetRef = AddInteractiveNodeWithRef(
          group,
          refRegistry,
          "target",
          $"Deck Position {i + 1}",
          new RefCallbacks()
        );
        targetRefToPosition[targetRef] = new CardOrderSelectionTarget
        {
          CardOrderSelectionTargetClass = new CardOrderSelectionTargetClass { Deck = position },
        };
      }

      // Void target
      if (selector.View?.IncludeVoid == true)
      {
        var voidRef = AddInteractiveNodeWithRef(
          group,
          refRegistry,
          "target",
          "Void",
          new RefCallbacks()
        );
        targetRefToPosition[voidRef] = new CardOrderSelectionTarget
        {
          Enum = CardOrderSelectionTargetEnum.Void,
        };
      }

      // Walk deck cards
      var deckGroup = CreateGroupNode($"Deck ({deckCount} cards)");
      for (var i = 0; i < selector.Objects.Count; i++)
      {
        var cardNode = BuildOrderSelectorCardNode(
          selector.Objects[i],
          $"deck position {i + 1}",
          targetRefToPosition,
          refRegistry
        );
        if (cardNode != null)
        {
          deckGroup.Children.Add(cardNode);
        }
      }
      if (deckGroup.Children.Count > 0)
      {
        group.Children.Add(deckGroup);
      }

      // Walk void cards
      var voidObjects = layout.CardOrderSelectorVoid.Objects;
      if (voidObjects.Count > 0)
      {
        var voidGroup = CreateGroupNode($"Void ({voidObjects.Count} cards)");
        foreach (var obj in voidObjects)
        {
          var cardNode = BuildOrderSelectorCardNode(obj, "void", targetRefToPosition, refRegistry);
          if (cardNode != null)
          {
            voidGroup.Children.Add(cardNode);
          }
        }
        if (voidGroup.Children.Count > 0)
        {
          group.Children.Add(voidGroup);
        }
      }

      return group.Children.Count > 0 ? group : null;
    }

    AbuSceneNode? BuildOrderSelectorCardNode(
      Displayable displayable,
      string locationLabel,
      Dictionary<string, CardOrderSelectionTarget> targetRefToPosition,
      RefRegistry refRegistry
    )
    {
      if (displayable is not Card card)
      {
        return null;
      }

      var revealed = card.CardView.Revealed;
      if (revealed == null)
      {
        return null;
      }

      var cardId = revealed.Actions.CanSelectOrder;
      if (cardId == null)
      {
        return null;
      }

      var name = ToSingleLineText(revealed.Name, fallback: "Unknown");
      var cardType = StripRichText(revealed.CardType);
      var label = !string.IsNullOrEmpty(cardType)
        ? $"{name}, {cardType} (at {locationLabel})"
        : $"{name} (at {locationLabel})";

      var node = CreateButtonNode(label);

      var callbacks = BuildDisplayableCallbacks(card);
      var capturedCardId = cardId.Value;
      callbacks.OnDrag = targetRef =>
      {
        if (targetRef != null && targetRefToPosition.TryGetValue(targetRef, out var target))
        {
          var action = new GameAction
          {
            GameActionClass = new()
            {
              BattleAction = new()
              {
                BattleActionClass = new()
                {
                  SelectOrderForDeckCard = new DeckCardSelectedOrder
                  {
                    CardId = capturedCardId,
                    Target = target,
                  },
                },
              },
            },
          };
          _registry.ActionService.PerformAction(action);
        }
      };
      refRegistry.Register(callbacks);
      return node;
    }

    // ── Object layout group helper ────────────────────────────────────

    AbuSceneNode? WalkObjectLayoutGroup(string label, ObjectLayout layout, RefRegistry refRegistry)
    {
      return WalkCardsGroup(label, label, layout.Objects, refRegistry);
    }
  }
}
