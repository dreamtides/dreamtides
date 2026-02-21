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
  /// snapshot tree. In battle mode, produces a zone-based hierarchy with
  /// player status, hand, battlefield, and action groups. Outside battle,
  /// falls back to a flat UIToolkit + 3D walk.
  /// </summary>
  public class DreamtidesSceneWalker : ISceneWalker
  {
    static readonly Regex RichTextTagPattern = new("<[^>]+>", RegexOptions.Compiled);

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
        root.Children.Add(WalkScene3DFallback(refRegistry));
      }

      return root;
    }

    // ── Battle mode ──────────────────────────────────────────────────

    AbuSceneNode WalkBattle(RefRegistry refRegistry)
    {
      var region = new AbuSceneNode { Role = "region", Label = "Battle" };
      var browserButtons = BuildBrowserButtonMap();
      var hasOpenPanels = _registry.DocumentService.HasOpenPanels;

      region.Children.Add(WalkControls(refRegistry));

      if (!hasOpenPanels)
      {
        region.Children.Add(WalkPlayer(
          refRegistry, browserButtons, "User", isUser: true));
        region.Children.Add(WalkPlayer(
          refRegistry, browserButtons, "Opponent", isUser: false));
        region.Children.Add(WalkActionButtons(refRegistry));
        AddStacks(region, refRegistry);
        AddGameModifiers(region, refRegistry);
        AddEssenceLabel(region);
        AddThinkingIndicator(region);
      }

      var uiOverlay = WalkUiToolkit(refRegistry);
      if (uiOverlay.Children.Count > 0)
      {
        region.Children.Add(uiOverlay);
      }

      return region;
    }

    // ── Controls ─────────────────────────────────────────────────────

    AbuSceneNode WalkControls(RefRegistry refRegistry)
    {
      var group = new AbuSceneNode { Role = "group", Label = "Controls" };
      var doc = _registry.DocumentService;
      TryAddCanvasButton(group, refRegistry, doc.MenuButton, "Menu");
      TryAddCanvasButton(group, refRegistry, doc.UndoButton, "Undo");
      TryAddCanvasButton(group, refRegistry, doc.DevButton, "Dev");
      TryAddCanvasButton(group, refRegistry, doc.BugButton, "Bug Report");
      return group;
    }

    void TryAddCanvasButton(
      AbuSceneNode parent, RefRegistry refRegistry, CanvasButton? button, string label)
    {
      if (button == null || !button.gameObject.activeSelf || button._canvasGroup.alpha <= 0)
      {
        return;
      }

      var node = new AbuSceneNode { Role = "button", Label = label, Interactive = true };
      refRegistry.Register(BuildCanvasButtonCallbacks(button));
      parent.Children.Add(node);
    }

    // ── Player ───────────────────────────────────────────────────────

    AbuSceneNode WalkPlayer(
      RefRegistry refRegistry,
      Dictionary<CardBrowserType, CardBrowserButton> browserButtons,
      string label,
      bool isUser)
    {
      var group = new AbuSceneNode { Role = "group", Label = label };
      var layout = _registry.BattleLayout;

      group.Children.Add(WalkStatus(isUser));
      AddBrowserButtons(group, refRegistry, browserButtons, isUser);
      group.Children.Add(WalkBattlefield(refRegistry, isUser));

      if (isUser)
      {
        AddDreamwell(group, refRegistry, layout.UserDreamwell);
        group.Children.Add(WalkHand(refRegistry, layout.UserHand.Objects, isUser: true));
      }
      else
      {
        AddDreamwell(group, refRegistry, layout.EnemyDreamwell);
        AddEnemyHandCount(group, layout.EnemyHand);
      }

      return group;
    }

    // ── Status ───────────────────────────────────────────────────────

    AbuSceneNode WalkStatus(bool isUser)
    {
      var group = new AbuSceneNode { Role = "group", Label = "Status" };
      var layout = _registry.BattleLayout;
      var status = isUser ? layout.UserStatusDisplay : layout.EnemyStatusDisplay;

      AddLabel(group, "Energy: " + StripRichText(status._energy._originalText ?? ""));
      AddLabel(group, "Score: " + StripRichText(status._score._originalText ?? ""));
      AddLabel(group, "Spark: " + StripRichText(status._totalSpark._originalText ?? ""));

      if (isUser)
      {
        if (status._leftTurnIndicator.activeSelf || status._rightTurnIndicator.activeSelf)
        {
          AddLabel(group, "Turn: yours");
        }
      }
      else
      {
        if (status._leftTurnIndicator.activeSelf || status._rightTurnIndicator.activeSelf)
        {
          AddLabel(group, "Turn: opponent's");
        }
      }

      return group;
    }

    // ── Browser buttons ──────────────────────────────────────────────

    static Dictionary<CardBrowserType, CardBrowserButton> BuildBrowserButtonMap()
    {
      var result = new Dictionary<CardBrowserType, CardBrowserButton>();
      var buttons = Object.FindObjectsByType<CardBrowserButton>(
        FindObjectsInactive.Exclude, FindObjectsSortMode.None);
      foreach (var button in buttons)
      {
        result[button._type] = button;
      }

      return result;
    }

    void AddBrowserButtons(
      AbuSceneNode parent,
      RefRegistry refRegistry,
      Dictionary<CardBrowserType, CardBrowserButton> browserButtons,
      bool isUser)
    {
      var layout = _registry.BattleLayout;

      var deckType = isUser ? CardBrowserType.UserDeck : CardBrowserType.EnemyDeck;
      var voidType = isUser ? CardBrowserType.UserVoid : CardBrowserType.EnemyVoid;
      var identityType = isUser ? CardBrowserType.UserStatus : CardBrowserType.EnemyStatus;
      var deckLayout = isUser ? layout.UserDeck : layout.EnemyDeck;
      var voidLayout = isUser ? layout.UserVoid : layout.EnemyVoid;

      AddZoneBrowserButton(parent, refRegistry, browserButtons, deckType,
        deckLayout.Objects.Count, "Deck");
      AddIdentityBrowserButton(parent, refRegistry, browserButtons, identityType);
      AddZoneBrowserButton(parent, refRegistry, browserButtons, voidType,
        voidLayout.Objects.Count, "Void");
    }

    void AddZoneBrowserButton(
      AbuSceneNode parent,
      RefRegistry refRegistry,
      Dictionary<CardBrowserType, CardBrowserButton> browserButtons,
      CardBrowserType type,
      int count,
      string zoneName)
    {
      var cardWord = count == 1 ? "card" : "cards";
      if (count > 0 && browserButtons.TryGetValue(type, out var button))
      {
        var node = new AbuSceneNode
        {
          Role = "button",
          Label = $"Browse {zoneName} ({count} {cardWord})",
          Interactive = true,
        };
        refRegistry.Register(BuildDisplayableCallbacks(button));
        parent.Children.Add(node);
      }
      else
      {
        AddLabel(parent, $"{zoneName}: {count} {cardWord}");
      }
    }

    void AddIdentityBrowserButton(
      AbuSceneNode parent,
      RefRegistry refRegistry,
      Dictionary<CardBrowserType, CardBrowserButton> browserButtons,
      CardBrowserType type)
    {
      if (browserButtons.TryGetValue(type, out var button))
      {
        var node = new AbuSceneNode
        {
          Role = "button",
          Label = "Browse Identity",
          Interactive = true,
        };
        refRegistry.Register(BuildDisplayableCallbacks(button));
        parent.Children.Add(node);
      }
    }

    // ── Battlefield ──────────────────────────────────────────────────

    AbuSceneNode WalkBattlefield(RefRegistry refRegistry, bool isUser)
    {
      var layout = _registry.BattleLayout;
      var objects = isUser
        ? layout.UserBattlefield.Objects
        : layout.EnemyBattlefield.Objects;

      var group = new AbuSceneNode { Role = "group", Label = "Battlefield" };

      foreach (var displayable in objects)
      {
        if (displayable is Card card)
        {
          var cardNode = BuildCardNode(card, refRegistry, isBattlefield: true);
          if (cardNode != null)
          {
            group.Children.Add(cardNode);
          }
        }
      }

      return group;
    }

    // ── Hand ─────────────────────────────────────────────────────────

    AbuSceneNode WalkHand(
      RefRegistry refRegistry, IReadOnlyList<Displayable> objects, bool isUser)
    {
      var cardWord = objects.Count == 1 ? "card" : "cards";
      var group = new AbuSceneNode
      {
        Role = "group",
        Label = $"Hand ({objects.Count} {cardWord})",
      };

      foreach (var displayable in objects)
      {
        if (displayable is Card card)
        {
          var cardNode = BuildCardNode(card, refRegistry, isBattlefield: false);
          if (cardNode != null)
          {
            group.Children.Add(cardNode);
          }
        }
      }

      return group;
    }

    void AddEnemyHandCount(AbuSceneNode parent, ObjectLayout enemyHand)
    {
      var count = enemyHand.Objects.Count;
      if (count > 0)
      {
        var cardWord = count == 1 ? "card" : "cards";
        AddLabel(parent, $"Hand: {count} {cardWord}");
      }
    }

    // ── Card node building ───────────────────────────────────────────

    AbuSceneNode? BuildCardNode(Card card, RefRegistry refRegistry, bool isBattlefield)
    {
      if (!card.CanHandleMouseEvents())
      {
        return null;
      }

      var revealed = card.CardView.Revealed;
      if (revealed == null)
      {
        return null;
      }

      var label = BuildCardLabel(revealed, isBattlefield);
      var node = new AbuSceneNode { Role = "button", Label = label, Interactive = true };
      refRegistry.Register(BuildDisplayableCallbacks(card));
      return node;
    }

    static string BuildCardLabel(RevealedCardView revealed, bool isBattlefield)
    {
      var name = StripRichText(revealed.Name ?? "").Replace("\n", ", ");
      var cardType = StripRichText(revealed.CardType ?? "");

      var annotations = new List<string>();
      if (revealed.Actions?.CanPlay != null)
      {
        annotations.Add("drag to play");
      }

      if (revealed.Actions?.OnClick != null)
      {
        annotations.Add("click to select");
      }

      if (!isBattlefield && revealed.Cost != null)
      {
        annotations.Add("cost: " + StripRichText(revealed.Cost));
      }

      if (isBattlefield && revealed.Spark != null)
      {
        annotations.Add("spark: " + StripRichText(revealed.Spark));
      }

      var suffix = annotations.Count > 0 ? " (" + string.Join(", ", annotations) + ")" : "";
      return string.IsNullOrEmpty(cardType)
        ? name + suffix
        : name + ", " + cardType + suffix;
    }

    // ── Action buttons ───────────────────────────────────────────────

    AbuSceneNode WalkActionButtons(RefRegistry refRegistry)
    {
      var group = new AbuSceneNode { Role = "group", Label = "Actions" };
      var layout = _registry.BattleLayout;
      TryAddActionButton(group, refRegistry, layout.PrimaryActionButton);
      TryAddActionButton(group, refRegistry, layout.SecondaryActionButton);
      TryAddActionButton(group, refRegistry, layout.IncrementActionButton);
      TryAddActionButton(group, refRegistry, layout.DecrementActionButton);
      return group;
    }

    void TryAddActionButton(AbuSceneNode parent, RefRegistry refRegistry, ActionButton button)
    {
      if (!button.gameObject.activeSelf
          || !button._text.gameObject.activeSelf
          || !button._collider.enabled)
      {
        return;
      }

      var label = StripRichText(button._text.text ?? button.gameObject.name);
      var node = new AbuSceneNode { Role = "button", Label = label, Interactive = true };
      refRegistry.Register(BuildDisplayableCallbacks(button));
      parent.Children.Add(node);
    }

    // ── Stacks ───────────────────────────────────────────────────────

    void AddStacks(AbuSceneNode parent, RefRegistry refRegistry)
    {
      var layout = _registry.BattleLayout;
      var stackObjects = new List<Displayable>();
      stackObjects.AddRange(layout.DefaultStack.Objects);
      stackObjects.AddRange(layout.TargetingUserStack.Objects);
      stackObjects.AddRange(layout.TargetingEnemyStack.Objects);
      stackObjects.AddRange(layout.TargetingBothStack.Objects);

      if (stackObjects.Count == 0)
      {
        return;
      }

      var group = new AbuSceneNode { Role = "group", Label = "Stack" };
      foreach (var displayable in stackObjects)
      {
        if (displayable is Card card)
        {
          var cardNode = BuildCardNode(card, refRegistry, isBattlefield: false);
          if (cardNode != null)
          {
            group.Children.Add(cardNode);
          }
        }
      }

      if (group.Children.Count > 0)
      {
        parent.Children.Add(group);
      }
    }

    // ── Game Modifiers ───────────────────────────────────────────────

    void AddGameModifiers(AbuSceneNode parent, RefRegistry refRegistry)
    {
      var objects = _registry.BattleLayout.GameModifiersDisplay.Objects;
      if (objects.Count == 0)
      {
        return;
      }

      var group = new AbuSceneNode { Role = "group", Label = "Game Modifiers" };
      foreach (var displayable in objects)
      {
        if (displayable is Card card)
        {
          var cardNode = BuildCardNode(card, refRegistry, isBattlefield: false);
          if (cardNode != null)
          {
            group.Children.Add(cardNode);
          }
        }
      }

      if (group.Children.Count > 0)
      {
        parent.Children.Add(group);
      }
    }

    // ── Dreamwell ────────────────────────────────────────────────────

    void AddDreamwell(AbuSceneNode parent, RefRegistry refRegistry, ObjectLayout dreamwell)
    {
      if (dreamwell.Objects.Count == 0)
      {
        return;
      }

      var group = new AbuSceneNode { Role = "group", Label = "Dreamwell" };
      foreach (var displayable in dreamwell.Objects)
      {
        if (displayable is Card card)
        {
          var cardNode = BuildCardNode(card, refRegistry, isBattlefield: false);
          if (cardNode != null)
          {
            group.Children.Add(cardNode);
          }
        }
      }

      if (group.Children.Count > 0)
      {
        parent.Children.Add(group);
      }
    }

    // ── Essence ──────────────────────────────────────────────────────

    void AddEssenceLabel(AbuSceneNode parent)
    {
      var essenceTotal = _registry.DreamscapeLayout.EssenceTotal;
      var text = essenceTotal._originalText;
      if (!string.IsNullOrEmpty(text))
      {
        AddLabel(parent, "Essence: " + StripRichText(text));
      }
    }

    // ── Thinking indicator ───────────────────────────────────────────

    void AddThinkingIndicator(AbuSceneNode parent)
    {
      if (_registry.BattleLayout.ThinkingIndicator.activeSelf)
      {
        AddLabel(parent, "Opponent is thinking...");
      }
    }

    // ── UIToolkit fallback ───────────────────────────────────────────

    AbuSceneNode WalkUiToolkit(RefRegistry refRegistry)
    {
      var region = new AbuSceneNode { Role = "region", Label = "UIToolkit" };
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
        node.Label ??= element.name;
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
      switch (element)
      {
        case NodeLabel label when !string.IsNullOrEmpty(label.text):
          return label.text;
        case NodeTypewriterText typewriter when !string.IsNullOrEmpty(typewriter.text):
          return typewriter.text;
        case NodeTextField textField when !string.IsNullOrEmpty(textField.value):
          return textField.value;
        case NodeSlider slider when !string.IsNullOrEmpty(slider.label):
          return slider.label;
        default:
          return !string.IsNullOrEmpty(element.name) ? element.name : null;
      }
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

    // ── Scene3D fallback (non-battle) ────────────────────────────────

    AbuSceneNode WalkScene3DFallback(RefRegistry refRegistry)
    {
      var region = new AbuSceneNode { Role = "region", Label = "Scene3D" };
      WalkDisplayables(region, refRegistry);
      WalkCanvasButtonsFallback(region, refRegistry);
      return region;
    }

    void WalkDisplayables(AbuSceneNode parent, RefRegistry refRegistry)
    {
      var displayables = Object.FindObjectsByType<Displayable>(
        FindObjectsInactive.Exclude, FindObjectsSortMode.None);

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

        refRegistry.Register(BuildDisplayableCallbacks(displayable));
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

    void WalkCanvasButtonsFallback(AbuSceneNode parent, RefRegistry refRegistry)
    {
      var doc = _registry.DocumentService;
      TryAddCanvasButtonFallback(parent, refRegistry, doc.MenuButton);
      TryAddCanvasButtonFallback(parent, refRegistry, doc.UndoButton);
      TryAddCanvasButtonFallback(parent, refRegistry, doc.DevButton);
      TryAddCanvasButtonFallback(parent, refRegistry, doc.BugButton);
    }

    void TryAddCanvasButtonFallback(
      AbuSceneNode parent, RefRegistry refRegistry, CanvasButton? button)
    {
      if (button == null || !button.gameObject.activeSelf || button._canvasGroup.alpha <= 0)
      {
        return;
      }

      var label = button._text.text;
      var node = new AbuSceneNode { Role = "button", Label = label, Interactive = true };
      refRegistry.Register(BuildCanvasButtonCallbacks(button));
      parent.Children.Add(node);
    }

    // ── Callback builders ────────────────────────────────────────────

    RefCallbacks BuildDisplayableCallbacks(Displayable displayable)
    {
      var callbacks = new RefCallbacks
      {
        OnClick = () =>
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
        },
        OnHover = () => { displayable.MouseHoverStart(); },
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

    // ── Utilities ────────────────────────────────────────────────────

    static void AddLabel(AbuSceneNode parent, string text)
    {
      parent.Children.Add(new AbuSceneNode { Role = "label", Label = text });
    }

    static string StripRichText(string? text)
    {
      if (string.IsNullOrEmpty(text))
      {
        return "";
      }

      var stripped = RichTextTagPattern.Replace(text, "");
      var chars = new List<char>(stripped.Length);
      for (var i = 0; i < stripped.Length; i++)
      {
        var c = stripped[i];
        // Filter Unicode Private Use Area characters (icon fonts)
        if (c >= 0xE000 && c <= 0xF8FF)
        {
          continue;
        }

        // Filter supplementary PUA (surrogate pairs)
        if (char.IsHighSurrogate(c) && i + 1 < stripped.Length && char.IsLowSurrogate(stripped[i + 1]))
        {
          var codePoint = char.ConvertToUtf32(c, stripped[i + 1]);
          if (codePoint >= 0xF0000 && codePoint <= 0xFFFFF)
          {
            i++; // skip low surrogate
            continue;
          }
        }

        chars.Add(c);
      }

      return new string(chars.ToArray()).Trim();
    }

    // ── Click simulation ─────────────────────────────────────────────

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
