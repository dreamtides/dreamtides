#nullable enable

using System;
using System.Collections;
using System.Collections.Generic;
using System.Runtime.CompilerServices;
using DG.Tweening;
using Dreamtides.Buttons;
using Dreamtides.Layout;
using Dreamtides.Masonry;
using Dreamtides.Schema;
using Dreamtides.Utils;
using UnityEngine;
using UnityEngine.EventSystems;
using UnityEngine.UIElements;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Services
{
  public class DocumentService : Service
  {
    [SerializeField]
    internal UIDocument _document = null!;
    IMasonElement _infoZoom = null!;
    IMasonElement _screenOverlay = null!;
    IMasonElement _screenAnchoredNode = null!;
    IMasonElement _effectPreviewOverlay = null!;
    Coroutine? _screenAnchorAutoHideCoroutine;

    [SerializeField]
    internal CanvasButton _menuButton = null!;
    public CanvasButton MenuButton => _menuButton;

    [SerializeField]
    internal CanvasButton _undoButton = null!;
    public CanvasButton UndoButton => _undoButton;

    [SerializeField]
    internal CanvasButton _devButton = null!;
    public CanvasButton DevButton => _devButton;

    [SerializeField]
    internal CanvasButton _bugButton = null!;
    public CanvasButton BugButton => _bugButton;

    const float ScreenAnchorFadeDurationSeconds = 0.3f;

    public FlexNode? CurrentScreenOverlayNode { get; private set; }

    public VisualElement RootVisualElement => _document.rootVisualElement;

    public bool HasOpenPanels { get; set; } = false;

    protected override void OnInitialize(GameMode _mode, TestConfiguration? testConfiguration)
    {
      _document.rootVisualElement.Clear();
      AddChild("InfoZoomContainer", out _infoZoom);
      AddChild("ScreenOverlay", out _screenOverlay);
      AddChild("ScreenAnchoredNode", out _screenAnchoredNode);
      AddChild("EffectPreviewOverlay", out _effectPreviewOverlay);
    }

    public bool IsPointerOverScreenElement()
    {
      return EventSystem.current.IsPointerOverGameObject();
    }

    public bool MouseOverDocumentElement()
    {
      foreach (var node in _screenOverlay.Self.Children())
      {
        var pointer = ScreenPositionToElementPosition(Registry.InputService.PointerPosition());
        if (node.ContainsPoint(pointer))
        {
          return true;
        }
      }

      return false;
    }

    /// <summary>
    /// Returns the GameContext values which are currently allowed for
    /// mouse/touch events to be handled. Displayable objects without a
    /// matching GameContext will ignore events. Returns null if all events
    /// are currently valid.
    /// </summary>
    public HashSet<GameContext>? AllowedContextForClicks()
    {
      if (Registry.Layout.Browser.Objects.Count > 0)
      {
        return new HashSet<GameContext>
        {
          GameContext.Browser,
          GameContext.Hand,
          GameContext.PrimaryActionButton,
        };
      }

      return null;
    }

    /// <summary>
    /// Scales a value in screen pixels to a value in element pixels.
    /// </summary>
    public float ScreenPxToElementPx(float value)
    {
      /// Dreamtides uses a 'scale with screen size' UI rendering system, with a
      /// reference resolution of 225x400 (16:9) and the matching mode set to
      /// 'height'.
      return value * (400f / Screen.height);
    }

    /// <summary>
    /// Converts a position from Screen coordinates to Element coordinates.
    ///
    /// Screen space is defined in pixels. The lower left pixel of the screen
    /// is (0, 0). The upper right pixel of the screen is
    /// (screen width in pixels - 1, screen height in pixels - 1).
    ///
    /// Element space is defined in density-independent 'pixels' based on a
    /// given reference DPI with (0, 0) appearing in the top right corner of the
    /// screen.
    /// </summary>
    public Vector2 ScreenPositionToElementPosition(Vector2 screenPosition) =>
      new(
        ScreenPxToElementPx(screenPosition.x),
        ScreenPxToElementPx(Screen.height - screenPosition.y)
      );

    /// <summary>
    /// Sets the active state of the main buttons in the document.
    /// </summary>
    public void SetMainButtonsActive(bool active)
    {
      _menuButton.gameObject.SetActive(active);
      _undoButton.gameObject.SetActive(active);
      _devButton.gameObject.SetActive(active);
      _bugButton.gameObject.SetActive(active);
    }

    public void FadeOutMainButtons(Sequence sequence)
    {
      sequence.Insert(atPosition: 0, TweenUtils.FadeOutCanvasGroup(_menuButton.CanvasGroup));
      sequence.Insert(atPosition: 0, TweenUtils.FadeOutCanvasGroup(_undoButton.CanvasGroup));
      sequence.Insert(atPosition: 0, TweenUtils.FadeOutCanvasGroup(_devButton.CanvasGroup));
      sequence.Insert(atPosition: 0, TweenUtils.FadeOutCanvasGroup(_bugButton.CanvasGroup));
      sequence.AppendCallback(() => SetMainButtonsActive(active: false));
    }

    public void FadeInMainButtons(Sequence sequence)
    {
      SetMainButtonsActive(active: true);
      sequence.Insert(atPosition: 0, TweenUtils.FadeInCanvasGroup(_menuButton.CanvasGroup));
      sequence.Insert(atPosition: 0, TweenUtils.FadeInCanvasGroup(_undoButton.CanvasGroup));
      sequence.Insert(atPosition: 0, TweenUtils.FadeInCanvasGroup(_devButton.CanvasGroup));
      sequence.Insert(atPosition: 0, TweenUtils.FadeInCanvasGroup(_bugButton.CanvasGroup));
    }

    public void RenderScreenOverlay(FlexNode? node)
    {
      CurrentScreenOverlayNode = node;
      Reconcile(ref _screenOverlay, node ?? new FlexNode());
    }

    public void RenderInfoZoom(FlexNode node)
    {
      Reconcile(ref _infoZoom, node);
    }

    public void RenderEffectPreview(FlexNode? node)
    {
      Reconcile(ref _effectPreviewOverlay, node ?? new FlexNode());
    }

    public void ClearInfoZoom()
    {
      Reconcile(ref _infoZoom, new FlexNode());
    }

    public void RenderScreenAnchoredNode(AnchorToScreenPositionCommand command)
    {
      if (command.Node == null)
      {
        Reconcile(ref _screenAnchoredNode, new FlexNode());
        if (_screenAnchorAutoHideCoroutine != null)
        {
          StopCoroutine(_screenAnchorAutoHideCoroutine);
          _screenAnchorAutoHideCoroutine = null;
        }
        return;
      }

      var position = TransformPositionToElementPosition(
        TransformForScreenAnchor(command.Anchor),
        Camera.main
      );
      var node = Mason.Row(
        "ScreenAnchorPosition",
        new FlexStyle
        {
          Position = FlexPosition.Absolute,
          Inset = new FlexInsets() { Left = Mason.Px(position.x), Top = Mason.Px(position.y) },
        },
        command.Node
      );

      Reconcile(ref _screenAnchoredNode, node);

      if (command.ShowDuration != null && command.ShowDuration.MillisecondsValue > 0)
      {
        StartAutoHideScreenAnchor(command.ShowDuration.ToSeconds());
      }
    }

    void StartAutoHideScreenAnchor(float delaySeconds)
    {
      if (_screenAnchorAutoHideCoroutine != null)
      {
        StopCoroutine(_screenAnchorAutoHideCoroutine);
      }
      _screenAnchorAutoHideCoroutine = StartCoroutine(ScreenAnchorAutoHideCoroutine(delaySeconds));
    }

    IEnumerator ScreenAnchorAutoHideCoroutine(float delaySeconds)
    {
      var elementAtSchedule = _screenAnchoredNode;
      yield return new WaitForSeconds(delaySeconds);

      // If another render replaced this node, abort.
      if (elementAtSchedule != _screenAnchoredNode)
      {
        yield break;
      }

      var ve = elementAtSchedule.Self;
      var startingOpacity = ve.resolvedStyle.opacity;
      var elapsed = 0f;
      while (elapsed < ScreenAnchorFadeDurationSeconds)
      {
        if (elementAtSchedule != _screenAnchoredNode)
        {
          yield break;
        }
        var t = elapsed / ScreenAnchorFadeDurationSeconds;
        ve.style.opacity = Mathf.Lerp(startingOpacity, 0f, t);
        elapsed += Time.deltaTime;
        yield return null;
      }

      if (elementAtSchedule == _screenAnchoredNode)
      {
        ve.style.opacity = 0f;
        Reconcile(ref _screenAnchoredNode, new FlexNode());
      }
    }

    Transform TransformForScreenAnchor(ScreenAnchor anchor)
    {
      if (anchor.SiteCharacter != null)
      {
        return Registry.DreamscapeService.CharacterScreenAnchorPosition(anchor.SiteCharacter);
      }

      throw new InvalidOperationException($"Unknown screen anchor: {anchor}");
    }

    Vector2 TransformPositionToElementPosition(Transform transform, Camera camera)
    {
      // Convert a world-space transform position into UI Toolkit panel coordinates.
      // 1. World -> Screen (bottom-left origin, pixels)
      // 2. Screen -> Element (top-left origin, scaled logical pixels) using existing helper
      // If the point is behind the camera (z < 0) we return an off-screen sentinel.
      if (transform == null || camera == null)
      {
        return Vector2.zero;
      }

      var screenPoint3 = camera.WorldToScreenPoint(transform.position);

      if (screenPoint3.z < 0f)
      {
        return new Vector2(-10000f, -10000f);
      }

      var screenPoint2 = new Vector2(screenPoint3.x, screenPoint3.y);
      return ScreenPositionToElementPosition(screenPoint2);
    }

    public DimensionGroup GetSafeArea()
    {
      var panel = RootVisualElement.panel;
      // Need to always use UnityEngine.Device to work properly in device
      // simulator.
      var safeLeftTop = RuntimePanelUtils.ScreenToPanel(
        panel,
        new Vector2(
          UnityEngine.Device.Screen.safeArea.xMin,
          UnityEngine.Device.Screen.height - UnityEngine.Device.Screen.safeArea.yMax
        )
      );
      var safeRightBottom = RuntimePanelUtils.ScreenToPanel(
        panel,
        new Vector2(
          UnityEngine.Device.Screen.width - UnityEngine.Device.Screen.safeArea.xMax,
          UnityEngine.Device.Screen.safeArea.yMin
        )
      );

      return Mason.GroupPx(
        top: safeLeftTop.y,
        right: safeRightBottom.x,
        bottom: safeRightBottom.y,
        left: safeLeftTop.x
      );
    }

    void AddChild(string elementName, out IMasonElement element)
    {
      var node = Mason.Row(
        elementName,
        new FlexStyle
        {
          Position = FlexPosition.Absolute,
          Inset = new FlexInsets()
          {
            Bottom = Mason.Px(0),
            Left = Mason.Px(0),
            Right = Mason.Px(0),
            Top = Mason.Px(0),
          },
          PickingMode = FlexPickingMode.Ignore,
        }
      );
      var container = MasonRenderer.Render(Registry, node);
      var result = new NodeVisualElement();
      container.Self.Add(result);
      element = result;
      _document.rootVisualElement.Add(container.Self);
    }

    void Reconcile(ref IMasonElement previousElement, FlexNode newNode)
    {
      var result = Reconciler.Update(Registry, newNode, previousElement);

      if (result != null)
      {
        previousElement = result;
      }
    }
  }
}
