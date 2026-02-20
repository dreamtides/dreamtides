#nullable enable

using System.Collections.Generic;
using Abu;
using Dreamtides.Buttons;
using Dreamtides.Components;
using Dreamtides.Layout;
using Dreamtides.Masonry;
using Dreamtides.Services;
using UnityEngine;
using UnityEngine.UIElements;

namespace Dreamtides.Abu
{
  /// <summary>
  /// Walks all three Dreamtides UI systems (UI Toolkit, 3D Displayables, and
  /// CanvasButtons) and produces an accessibility snapshot tree with click and
  /// hover callbacks registered in the ref registry.
  /// </summary>
  public class DreamtidesSceneWalker : ISceneWalker
  {
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

      root.Children.Add(WalkUiToolkit(refRegistry));
      root.Children.Add(WalkScene3D(refRegistry));

      return root;
    }

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
        node.Label ??= element.name;
      }

      // Always recurse into children
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
      // Draggable is a stub; mark as non-interactive
      if (element is Draggable)
      {
        return false;
      }

      return element.pickingMode == PickingMode.Position;
    }

    RefCallbacks BuildUiToolkitCallbacks(VisualElement element)
    {
      var callbacks = new RefCallbacks();

      // Click: Use direct Callbacks.OnClick() invocation (validated by Gate 0)
      if (element is INodeCallbacks nodeCallbacks)
      {
        var cb = nodeCallbacks.Callbacks.Value;
        callbacks.OnClick = () =>
        {
          using var clickEvent = ClickEvent.GetPooled();
          cb.OnClick(clickEvent);
        };

        // Hover: Use direct Callbacks.OnMouseEnter() invocation (validated by Gate 0)
        callbacks.OnHover = () =>
        {
          using var enterEvent = MouseEnterEvent.GetPooled();
          cb.OnMouseEnter(enterEvent);
        };
      }

      return callbacks;
    }

    AbuSceneNode WalkScene3D(RefRegistry refRegistry)
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

        var callbacks = BuildDisplayableCallbacks(displayable);
        refRegistry.Register(callbacks);

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

    RefCallbacks BuildDisplayableCallbacks(Displayable displayable)
    {
      var callbacks = new RefCallbacks();

      // Click: Inject IInputProvider and run two-frame click sequence
      // (validated by Gate 0)
      callbacks.OnClick = () =>
      {
        var originalProvider = _registry.InputService.InputProvider;
        var fakeInput = new DisplayableClickInputProvider(displayable);
        try
        {
          _registry.InputService.InputProvider = fakeInput;

          // Frame 1: pressed with target
          fakeInput.Phase = ClickPhase.Pressed;
          _registry.InputService.Update();

          // Frame 2: released with same target
          fakeInput.Phase = ClickPhase.Released;
          _registry.InputService.Update();
        }
        finally
        {
          _registry.InputService.InputProvider = originalProvider;
        }
      };

      // Hover: Call hover methods directly on the Displayable
      callbacks.OnHover = () =>
      {
        displayable.MouseHoverStart();
      };

      return callbacks;
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
      if (button == null)
      {
        return;
      }

      if (!button.gameObject.activeSelf || button._canvasGroup.alpha <= 0)
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

    static RefCallbacks BuildCanvasButtonCallbacks(CanvasButton button)
    {
      return new RefCallbacks
      {
        // Click: Call OnClick() directly (validated by Gate 0)
        OnClick = () => button.OnClick(),
        OnHover = () => button.MouseHoverStart(),
      };
    }

    /// <summary>
    /// Input provider used to simulate a click on a specific Displayable by
    /// injecting it into InputService.InputProvider for two frames.
    /// </summary>
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
