#nullable enable

using Abu;
using Dreamtides.Buttons;
using Dreamtides.Components;
using Dreamtides.Layout;
using Dreamtides.Masonry;
using UnityEngine;
using UnityEngine.UIElements;
using Object = UnityEngine.Object;

namespace Dreamtides.Abu
{
  public partial class DreamtidesSceneWalker
  {
    // ── UIToolkit (filtered for battle overlays) ──────────────────────

    AbuSceneNode? WalkUiToolkitFiltered(RefRegistry refRegistry)
    {
      var rootElement = GetRootVisualElement();
      if (rootElement == null)
      {
        return null;
      }

      var region = CreateRegionNode("UIToolkit");
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
      var region = CreateRegionNode("UIToolkit");
      var rootElement = GetRootVisualElement();
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
      var node = CreateNode(role, label, interactive);

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

    VisualElement? GetRootVisualElement()
    {
      return _registry.DocumentService._document != null
        ? _registry.DocumentService.RootVisualElement
        : null;
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
        NodeTypewriterText typewriter when !string.IsNullOrEmpty(typewriter.text) =>
          typewriter.text,
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

    // ── Fallback Scene3D (non-battle) ─────────────────────────────────

    AbuSceneNode WalkFallbackScene3D(RefRegistry refRegistry)
    {
      var region = CreateRegionNode("Scene3D");
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
        AddInteractiveNode(
          parent,
          refRegistry,
          "button",
          label,
          BuildDisplayableCallbacks(displayable)
        );
      }
    }

    static string DetermineDisplayableLabel(Displayable displayable)
    {
      switch (displayable)
      {
        case Card card:
          return ToSingleLineText(card.CardView.Revealed?.Name, fallback: "Hidden Card");
        case ActionButton actionButton:
          return ToSingleLineText(actionButton._text.text, fallback: displayable.gameObject.name);
        case DisplayableButton displayableButton:
          return ToSingleLineText(
            displayableButton._text.text,
            fallback: displayable.gameObject.name
          );
        case CardBrowserButton:
          return ToSingleLineText(displayable.gameObject.name);
        default:
          return ToSingleLineText(displayable.gameObject.name);
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

    void TryAddCanvasButton(
      AbuSceneNode parent,
      RefRegistry refRegistry,
      CanvasButton? button,
      string? labelOverride = null
    )
    {
      if (button == null || !button.gameObject.activeSelf || button._canvasGroup.alpha <= 0)
      {
        return;
      }

      var label = labelOverride ?? StripRichText(button._text.text);
      if (string.IsNullOrEmpty(label))
      {
        return;
      }

      AddInteractiveNode(parent, refRegistry, "button", label, BuildCanvasButtonCallbacks(button));
    }
  }
}
