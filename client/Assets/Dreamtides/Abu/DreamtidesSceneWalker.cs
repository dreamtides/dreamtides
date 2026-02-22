#nullable enable

using System.Collections.Generic;
using System.Text.RegularExpressions;
using Abu;
using Dreamtides.Buttons;
using Dreamtides.Components;
using Dreamtides.Layout;
using Dreamtides.Masonry;
using Dreamtides.Schema;
using Dreamtides.Services;
using UnityEngine;
using UnityEngine.UIElements;

namespace Dreamtides.Abu
{
  /// <summary>
  /// Walks all Dreamtides UI systems and produces a structured accessibility
  /// snapshot tree. In battle mode, the tree is organized by zone (hand,
  /// battlefield, etc.) with semantic labels. Outside battle, falls back to
  /// a flat UIToolkit + 3D walk.
  /// </summary>
  public class DreamtidesSceneWalker : ISceneWalker
  {
    static readonly Regex RichTextTagPattern = new("<[^>]+>");

    readonly Registry _registry;

    public DreamtidesSceneWalker(Registry registry)
    {
      _registry = registry;
    }

    public AbuSceneNode Walk(RefRegistry refRegistry)
    {
      var root = new AbuSceneNode
      {
        Role = "application",
        Label = "Dreamtides",
        Interactive = false,
      };

      if (_registry.BattleLayout.Contents.activeSelf)
      {
        root.Children.Add(WalkBattle(refRegistry));
      }
      else
      {
        root.Children.Add(WalkUiToolkit(refRegistry));
        root.Children.Add(WalkFallbackScene3D(refRegistry));
      }

      return root;
    }

    // ── Rich text stripping ───────────────────────────────────────────

    static string StripRichText(string? text)
    {
      if (string.IsNullOrEmpty(text))
      {
        return "";
      }

      var stripped = RichTextTagPattern.Replace(text, "");
      var sb = new System.Text.StringBuilder(stripped.Length);
      for (var i = 0; i < stripped.Length; i++)
      {
        var c = stripped[i];
        var code = (int)c;
        // Filter icon glyphs mapped to CJK code points by icon fonts
        if (code >= 0x3400 && code <= 0x9FFF)
        {
          continue;
        }

        // Filter icon glyphs (PUA + CJK compat / presentation forms / specials)
        if (code >= 0xE000 && code <= 0xFFFF)
        {
          continue;
        }

        // Filter Supplementary Private Use Area (represented as surrogate pairs)
        if (
          char.IsHighSurrogate(c)
          && i + 1 < stripped.Length
          && char.IsLowSurrogate(stripped[i + 1])
        )
        {
          var codePoint = char.ConvertToUtf32(c, stripped[i + 1]);
          if (codePoint >= 0xF0000 && codePoint <= 0xFFFFF)
          {
            i++; // skip low surrogate
            continue;
          }
        }

        sb.Append(c);
      }

      return sb.ToString().Trim();
    }

    // ── Battle mode walk ──────────────────────────────────────────────

    AbuSceneNode WalkBattle(RefRegistry refRegistry)
    {
      var region = new AbuSceneNode
      {
        Role = "region",
        Label = "Battle",
        Interactive = false,
      };
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

        // 7. Essence label
        AddEssenceLabel(region);

        // 8. Play zone (drag target for playing cards from hand)
        region.Children.Add(
          new AbuSceneNode
          {
            Role = "target",
            Label = "Play Zone",
            Interactive = true,
          }
        );
        refRegistry.Register(new RefCallbacks());

        // 9. Thinking indicator
        if (layout.ThinkingIndicator.activeSelf)
        {
          region.Children.Add(
            new AbuSceneNode
            {
              Role = "label",
              Label = "Opponent is thinking...",
              Interactive = false,
            }
          );
        }
      }

      // Card Order Selector (shown when active)
      var cardOrderGroup = WalkCardOrderSelector(layout, refRegistry);
      if (cardOrderGroup != null)
      {
        region.Children.Add(cardOrderGroup);
      }

      // 10. UI overlays (filtered, only when content exists)
      var uiOverlay = WalkUiToolkitFiltered(refRegistry);
      if (uiOverlay != null)
      {
        region.Children.Add(uiOverlay);
      }

      return region;
    }

    // ── Controls ──────────────────────────────────────────────────────

    AbuSceneNode WalkControls(RefRegistry refRegistry)
    {
      var group = new AbuSceneNode
      {
        Role = "group",
        Label = "Controls",
        Interactive = false,
      };
      var doc = _registry.DocumentService;

      TryAddCanvasButtonWithLabel(group, refRegistry, doc.MenuButton, "Menu");
      TryAddCanvasButtonWithLabel(group, refRegistry, doc.UndoButton, "Undo");
      TryAddCanvasButtonWithLabel(group, refRegistry, doc.DevButton, "Dev");
      TryAddCanvasButtonWithLabel(group, refRegistry, doc.BugButton, "Bug Report");
      TryAddCloseBrowserButton(group, refRegistry);

      return group;
    }

    void TryAddCanvasButtonWithLabel(
      AbuSceneNode parent,
      RefRegistry refRegistry,
      CanvasButton? button,
      string label
    )
    {
      if (button == null || !button.gameObject.activeSelf || button._canvasGroup.alpha <= 0)
      {
        return;
      }

      var node = new AbuSceneNode
      {
        Role = "button",
        Label = label,
        Interactive = true,
      };
      var callbacks = BuildCanvasButtonCallbacks(button);
      refRegistry.Register(callbacks);
      parent.Children.Add(node);
    }

    void TryAddCloseBrowserButton(AbuSceneNode parent, RefRegistry refRegistry)
    {
      var button = _registry.BattleLayout.CloseBrowserButton;
      if (!button.gameObject.activeSelf)
      {
        return;
      }

      var node = new AbuSceneNode
      {
        Role = "button",
        Label = "Close Browser",
        Interactive = true,
      };
      refRegistry.Register(
        new RefCallbacks
        {
          OnClick = () => button.OnClick(),
        }
      );
      parent.Children.Add(node);
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
      var group = new AbuSceneNode
      {
        Role = "group",
        Label = playerLabel,
        Interactive = false,
      };

      // Status
      group.Children.Add(WalkStatus(statusDisplay, isUser));

      // Deck browser
      AddBrowserButton(group, browserButtons, deckType, "Deck", refRegistry);

      // Identity browser
      AddBrowserButton(group, browserButtons, statusType, "Identity", refRegistry);

      // Void browser
      AddBrowserButton(group, browserButtons, voidType, "Void", refRegistry);

      // Battlefield
      var battlefieldGroup = new AbuSceneNode
      {
        Role = "group",
        Label = "Battlefield",
        Interactive = false,
      };
      foreach (var obj in battlefield.Objects)
      {
        var cardNode = BuildCardNode(obj, "Battlefield", refRegistry);
        if (cardNode != null)
        {
          battlefieldGroup.Children.Add(cardNode);
        }
      }
      if (battlefieldGroup.Children.Count > 0)
      {
        group.Children.Add(battlefieldGroup);
      }

      // Hand
      if (isUser)
      {
        var handGroup = new AbuSceneNode
        {
          Role = "group",
          Label = $"Hand ({handObjects.Count} cards)",
          Interactive = false,
        };
        foreach (var obj in handObjects)
        {
          var cardNode = BuildCardNode(obj, "Hand", refRegistry);
          if (cardNode != null)
          {
            handGroup.Children.Add(cardNode);
          }
        }
        if (handGroup.Children.Count > 0)
        {
          group.Children.Add(handGroup);
        }
      }
      else if (handObjects.Count > 0)
      {
        group.Children.Add(
          new AbuSceneNode
          {
            Role = "label",
            Label = $"Hand: {handObjects.Count} cards",
            Interactive = false,
          }
        );
      }

      return group;
    }

    // ── Status ────────────────────────────────────────────────────────

    AbuSceneNode WalkStatus(PlayerStatusDisplay statusDisplay, bool isUser)
    {
      var group = new AbuSceneNode
      {
        Role = "group",
        Label = "Status",
        Interactive = false,
      };

      var energyText = StripRichText(statusDisplay._energy._originalText);
      if (!string.IsNullOrEmpty(energyText))
      {
        group.Children.Add(
          new AbuSceneNode
          {
            Role = "label",
            Label = $"Energy: {energyText}",
            Interactive = false,
          }
        );
      }

      var scoreText = StripRichText(statusDisplay._score._originalText);
      if (!string.IsNullOrEmpty(scoreText))
      {
        group.Children.Add(
          new AbuSceneNode
          {
            Role = "label",
            Label = $"Score: {scoreText}",
            Interactive = false,
          }
        );
      }

      var sparkText = StripRichText(statusDisplay._totalSpark._originalText);
      if (!string.IsNullOrEmpty(sparkText))
      {
        group.Children.Add(
          new AbuSceneNode
          {
            Role = "label",
            Label = $"Spark: {sparkText}",
            Interactive = false,
          }
        );
      }

      if (isUser)
      {
        var hasTurn =
          (statusDisplay._leftTurnIndicator != null && statusDisplay._leftTurnIndicator.activeSelf)
          || (
            statusDisplay._rightTurnIndicator != null
            && statusDisplay._rightTurnIndicator.activeSelf
          );
        group.Children.Add(
          new AbuSceneNode
          {
            Role = "label",
            Label = hasTurn ? "Turn: yours" : "Turn: opponent's",
            Interactive = false,
          }
        );
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

      if (zoneName == "Identity")
      {
        // Identity always shows as a button
        var node = new AbuSceneNode
        {
          Role = "button",
          Label = $"Browse {zoneName}",
          Interactive = true,
        };
        RegisterDisplayableCallbacks(button, refRegistry);
        parent.Children.Add(node);
        return;
      }

      if (count > 0)
      {
        var node = new AbuSceneNode
        {
          Role = "button",
          Label = $"Browse {zoneName} ({count} cards)",
          Interactive = true,
        };
        RegisterDisplayableCallbacks(button, refRegistry);
        parent.Children.Add(node);
      }
      else
      {
        parent.Children.Add(
          new AbuSceneNode
          {
            Role = "label",
            Label = $"{zoneName}: 0 cards",
            Interactive = false,
          }
        );
      }
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

      var revealed = card.CardView.Revealed;
      if (revealed == null)
      {
        return null;
      }

      var canPlay = zoneContext == "Hand"
        && revealed.Actions.CanPlay is { } cp
        && !cp.IsNull
        && _registry.CapabilitiesService.CanPlayCards();

      var label = BuildCardLabel(revealed, zoneContext, canPlay);
      var node = new AbuSceneNode
      {
        Role = "button",
        Label = label,
        Interactive = true,
      };
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
      var name = StripRichText(revealed.Name)?.Replace("\n", ", ") ?? "Unknown";
      var cardType = StripRichText(revealed.CardType);

      var annotations = new List<string>();

      if (zoneContext == "Hand" && !string.IsNullOrEmpty(revealed.Cost))
      {
        annotations.Add($"cost: {StripRichText(revealed.Cost)}");
      }

      if (zoneContext == "Battlefield" && !string.IsNullOrEmpty(revealed.Spark))
      {
        annotations.Add($"spark: {StripRichText(revealed.Spark)}");
      }

      if (canPlay)
      {
        annotations.Add("can play");
      }

      var suffix = annotations.Count > 0 ? $" ({string.Join(", ", annotations)})" : "";
      if (!string.IsNullOrEmpty(cardType))
      {
        return $"{name}, {cardType}{suffix}";
      }

      return $"{name}{suffix}";
    }

    // ── Action buttons ────────────────────────────────────────────────

    AbuSceneNode WalkActionButtons(BattleLayout layout, RefRegistry refRegistry)
    {
      var group = new AbuSceneNode
      {
        Role = "group",
        Label = "Actions",
        Interactive = false,
      };
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

      var node = new AbuSceneNode
      {
        Role = "button",
        Label = label,
        Interactive = true,
      };
      RegisterDisplayableCallbacks(button, refRegistry);
      parent.Children.Add(node);
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
        parent.Children.Add(
          new AbuSceneNode
          {
            Role = "label",
            Label = $"Essence: {essenceText}",
            Interactive = false,
          }
        );
      }
    }

    // ── Stack ─────────────────────────────────────────────────────────

    AbuSceneNode? WalkStack(BattleLayout layout, RefRegistry refRegistry)
    {
      var stackGroup = new AbuSceneNode
      {
        Role = "group",
        Label = "Stack",
        Interactive = false,
      };

      AddStackObjects(stackGroup, layout.DefaultStack, refRegistry);
      AddStackObjects(stackGroup, layout.TargetingUserStack, refRegistry);
      AddStackObjects(stackGroup, layout.TargetingEnemyStack, refRegistry);
      AddStackObjects(stackGroup, layout.TargetingBothStack, refRegistry);

      return stackGroup.Children.Count > 0 ? stackGroup : null;
    }

    void AddStackObjects(AbuSceneNode parent, ObjectLayout stack, RefRegistry refRegistry)
    {
      foreach (var obj in stack.Objects)
      {
        var cardNode = BuildCardNode(obj, "Stack", refRegistry);
        if (cardNode != null)
        {
          parent.Children.Add(cardNode);
        }
      }
    }

    // ── Card Order Selector ────────────────────────────────────────────

    AbuSceneNode? WalkCardOrderSelector(BattleLayout layout, RefRegistry refRegistry)
    {
      var selector = layout.CardOrderSelector;
      if (!selector.IsOpen)
      {
        return null;
      }

      var group = new AbuSceneNode
      {
        Role = "group",
        Label = "Card Order Selector",
        Interactive = false,
      };

      // Build target ref lookup so card OnDrag can map target refs to actions
      var targetRefToPosition = new Dictionary<string, CardOrderSelectionTarget>();

      // Deck position targets: 0..N for N cards currently in deck
      var deckCount = selector.Objects.Count;
      for (var i = 0; i <= deckCount; i++)
      {
        var position = i;
        var targetNode = new AbuSceneNode
        {
          Role = "target",
          Label = $"Deck Position {i + 1}",
          Interactive = true,
        };
        var targetRef = refRegistry.Register(new RefCallbacks());
        targetRefToPosition[targetRef] = new CardOrderSelectionTarget
        {
          CardOrderSelectionTargetClass = new CardOrderSelectionTargetClass { Deck = position },
        };
        group.Children.Add(targetNode);
      }

      // Void target
      if (selector.View?.IncludeVoid == true)
      {
        var voidNode = new AbuSceneNode
        {
          Role = "target",
          Label = "Void",
          Interactive = true,
        };
        var voidRef = refRegistry.Register(new RefCallbacks());
        targetRefToPosition[voidRef] = new CardOrderSelectionTarget
        {
          Enum = CardOrderSelectionTargetEnum.Void,
        };
        group.Children.Add(voidNode);
      }

      // Walk deck cards
      var deckGroup = new AbuSceneNode
      {
        Role = "group",
        Label = $"Deck ({deckCount} cards)",
        Interactive = false,
      };
      for (var i = 0; i < selector.Objects.Count; i++)
      {
        var cardNode = BuildOrderSelectorCardNode(
          selector.Objects[i], $"deck position {i + 1}", targetRefToPosition, refRegistry
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
        var voidGroup = new AbuSceneNode
        {
          Role = "group",
          Label = $"Void ({voidObjects.Count} cards)",
          Interactive = false,
        };
        foreach (var obj in voidObjects)
        {
          var cardNode = BuildOrderSelectorCardNode(
            obj, "void", targetRefToPosition, refRegistry
          );
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

      var name = StripRichText(revealed.Name)?.Replace("\n", ", ") ?? "Unknown";
      var cardType = StripRichText(revealed.CardType);
      var label = !string.IsNullOrEmpty(cardType)
        ? $"{name}, {cardType} (at {locationLabel})"
        : $"{name} (at {locationLabel})";

      var node = new AbuSceneNode
      {
        Role = "button",
        Label = label,
        Interactive = true,
      };

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
      var group = new AbuSceneNode
      {
        Role = "group",
        Label = label,
        Interactive = false,
      };
      foreach (var obj in layout.Objects)
      {
        var cardNode = BuildCardNode(obj, label, refRegistry);
        if (cardNode != null)
        {
          group.Children.Add(cardNode);
        }
      }

      return group.Children.Count > 0 ? group : null;
    }

    // ── Displayable callbacks ─────────────────────────────────────────

    void RegisterDisplayableCallbacks(Displayable displayable, RefRegistry refRegistry)
    {
      var callbacks = BuildDisplayableCallbacks(displayable);
      refRegistry.Register(callbacks);
    }

    RefCallbacks BuildDisplayableCallbacks(Displayable displayable)
    {
      var callbacks = new RefCallbacks();
      callbacks.OnClick = () =>
      {
        var originalProvider = _registry.InputService.InputProvider;
        var fakeInput = new DisplayableClickInputProvider(displayable);
        try
        {
          _registry.InputService.InputProvider = fakeInput;
          fakeInput.Phase = ClickPhase.Pressed;
          _registry.InputService.Update();
          fakeInput.Phase = ClickPhase.Released;
          _registry.InputService.Update();
        }
        finally
        {
          _registry.InputService.InputProvider = originalProvider;
        }
      };
      callbacks.OnHover = () =>
      {
        displayable.MouseHoverStart();
      };
      return callbacks;
    }

    static RefCallbacks BuildCanvasButtonCallbacks(CanvasButton button)
    {
      return new RefCallbacks
      {
        OnClick = () => button.OnClick(),
        OnHover = () => button.MouseHoverStart(),
      };
    }

    // ── UIToolkit (filtered for battle overlays) ──────────────────────

    AbuSceneNode? WalkUiToolkitFiltered(RefRegistry refRegistry)
    {
      var doc = _registry.DocumentService;
      var rootElement = doc._document != null ? doc.RootVisualElement : null;
      if (rootElement == null)
      {
        return null;
      }

      var region = new AbuSceneNode
      {
        Role = "region",
        Label = "UIToolkit",
        Interactive = false,
      };
      foreach (var child in rootElement.Children())
      {
        var childNode = WalkVisualElement(child, refRegistry);
        if (childNode != null && HasContent(childNode))
        {
          region.Children.Add(childNode);
        }
      }

      return region.Children.Count > 0 ? region : null;
    }

    static bool HasContent(AbuSceneNode node)
    {
      if (node.Interactive && !string.IsNullOrEmpty(node.Label))
      {
        return true;
      }

      if (node.Role == "label" && !string.IsNullOrEmpty(node.Label))
      {
        return true;
      }

      foreach (var child in node.Children)
      {
        if (HasContent(child))
        {
          return true;
        }
      }

      return false;
    }

    // ── UIToolkit full walk (fallback for non-battle) ─────────────────

    AbuSceneNode WalkUiToolkit(RefRegistry refRegistry)
    {
      var region = new AbuSceneNode
      {
        Role = "region",
        Label = "UIToolkit",
        Interactive = false,
      };
      var doc = _registry.DocumentService;
      var rootElement = doc._document != null ? doc.RootVisualElement : null;
      if (rootElement != null)
      {
        foreach (var child in rootElement.Children())
        {
          var childNode = WalkVisualElement(child, refRegistry);
          if (childNode != null)
          {
            region.Children.Add(childNode);
          }
        }
      }

      return region;
    }

    AbuSceneNode? WalkVisualElement(VisualElement element, RefRegistry refRegistry)
    {
      var role = DetermineRole(element);
      var label = DetermineLabel(element);
      var interactive = IsInteractive(element);

      var node = new AbuSceneNode
      {
        Role = role,
        Label = label,
        Interactive = interactive,
      };

      if (interactive)
      {
        var callbacks = BuildUiToolkitCallbacks(element);
        refRegistry.Register(callbacks);
        node.Label ??= StripRichText(element.name);
      }

      foreach (var child in element.Children())
      {
        var childNode = WalkVisualElement(child, refRegistry);
        if (childNode != null)
        {
          node.Children.Add(childNode);
        }
      }

      return node;
    }

    static string DetermineRole(VisualElement element)
    {
      return element switch
      {
        Draggable => "generic",
        NodeTextField => "textbox",
        NodeSlider => "slider",
        NodeLabel => "label",
        NodeTypewriterText => "label",
        NodeVisualElement when element.pickingMode == PickingMode.Position => "button",
        _ => "group",
      };
    }

    static string? DetermineLabel(VisualElement element)
    {
      string? raw = element switch
      {
        NodeLabel label when !string.IsNullOrEmpty(label.text) => label.text,
        NodeTypewriterText typewriter when !string.IsNullOrEmpty(typewriter.text) => typewriter.text,
        NodeTextField textField when !string.IsNullOrEmpty(textField.value) => textField.value,
        NodeSlider slider when !string.IsNullOrEmpty(slider.label) => slider.label,
        _ => !string.IsNullOrEmpty(element.name) ? element.name : null,
      };
      var stripped = StripRichText(raw);
      return string.IsNullOrEmpty(stripped) ? null : stripped;
    }

    static bool IsInteractive(VisualElement element)
    {
      if (element is Draggable)
      {
        return false;
      }

      return element.pickingMode == PickingMode.Position;
    }

    RefCallbacks BuildUiToolkitCallbacks(VisualElement element)
    {
      var callbacks = new RefCallbacks();
      if (element is INodeCallbacks nodeCallbacks)
      {
        var cb = nodeCallbacks.Callbacks.Value;
        callbacks.OnClick = () =>
        {
          using var clickEvent = ClickEvent.GetPooled();
          cb.OnClick(clickEvent);
        };
        callbacks.OnHover = () =>
        {
          using var enterEvent = MouseEnterEvent.GetPooled();
          cb.OnMouseEnter(enterEvent);
        };
      }

      return callbacks;
    }

    // ── Fallback Scene3D (non-battle) ─────────────────────────────────

    AbuSceneNode WalkFallbackScene3D(RefRegistry refRegistry)
    {
      var region = new AbuSceneNode
      {
        Role = "region",
        Label = "Scene3D",
        Interactive = false,
      };
      var hasOpenPanels = _registry.DocumentService.HasOpenPanels;

      if (!hasOpenPanels)
      {
        WalkDisplayables(region, refRegistry);
        WalkCanvasButtons(region, refRegistry);
      }

      return region;
    }

    void WalkDisplayables(AbuSceneNode parent, RefRegistry refRegistry)
    {
      var displayables = Object.FindObjectsByType<Displayable>(
        FindObjectsInactive.Exclude,
        FindObjectsSortMode.None
      );

      foreach (var displayable in displayables)
      {
        if (!displayable.CanHandleMouseEvents())
        {
          continue;
        }

        var label = DetermineDisplayableLabel(displayable);
        var node = new AbuSceneNode
        {
          Role = "button",
          Label = label,
          Interactive = true,
        };
        RegisterDisplayableCallbacks(displayable, refRegistry);
        parent.Children.Add(node);
      }
    }

    static string DetermineDisplayableLabel(Displayable displayable)
    {
      switch (displayable)
      {
        case Card card:
          return card.CardView.Revealed?.Name ?? "Hidden Card";
        case ActionButton actionButton:
          return actionButton._text.text ?? displayable.gameObject.name;
        case DisplayableButton displayableButton:
          return displayableButton._text.text ?? displayable.gameObject.name;
        case CardBrowserButton:
          return displayable.gameObject.name;
        default:
          return displayable.gameObject.name;
      }
    }

    void WalkCanvasButtons(AbuSceneNode parent, RefRegistry refRegistry)
    {
      var doc = _registry.DocumentService;
      TryAddCanvasButton(parent, refRegistry, doc.MenuButton);
      TryAddCanvasButton(parent, refRegistry, doc.UndoButton);
      TryAddCanvasButton(parent, refRegistry, doc.DevButton);
      TryAddCanvasButton(parent, refRegistry, doc.BugButton);
    }

    void TryAddCanvasButton(AbuSceneNode parent, RefRegistry refRegistry, CanvasButton? button)
    {
      if (button == null || !button.gameObject.activeSelf || button._canvasGroup.alpha <= 0)
      {
        return;
      }

      var label = button._text.text;
      var node = new AbuSceneNode
      {
        Role = "button",
        Label = label,
        Interactive = true,
      };
      var callbacks = BuildCanvasButtonCallbacks(button);
      refRegistry.Register(callbacks);
      parent.Children.Add(node);
    }

    // ── Input simulation ──────────────────────────────────────────────

    enum ClickPhase
    {
      Pressed,
      Released,
    }

    sealed class DisplayableClickInputProvider : IInputProvider
    {
      readonly Displayable _target;

      public ClickPhase Phase { get; set; } = ClickPhase.Pressed;

      public DisplayableClickInputProvider(Displayable target)
      {
        _target = target;
      }

      public bool IsPointerPressed() => Phase == ClickPhase.Pressed;

      public Vector2 PointerPosition() => Vector2.zero;

      public Displayable? ObjectAtPointerPosition(MouseEventType eventType)
      {
        return eventType switch
        {
          MouseEventType.MouseDown => _target,
          MouseEventType.MouseUp => _target,
          MouseEventType.MouseHover => null,
          _ => null,
        };
      }
    }
  }
}
