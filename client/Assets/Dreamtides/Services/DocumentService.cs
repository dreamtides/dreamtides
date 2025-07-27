#nullable enable

using System.Collections.Generic;
using Dreamtides.Layout;
using Dreamtides.Masonry;
using Dreamtides.Schema;
using UnityEngine;
using UnityEngine.EventSystems;
using UnityEngine.UIElements;

namespace Dreamtides.Services
{
  public class DocumentService : Service
  {
    [SerializeField] UIDocument _document = null!;
    IMasonElement _infoZoom = null!;
    IMasonElement _screenOverlay = null!;

    public FlexNode? CurrentScreenOverlayNode { get; private set; }

    public VisualElement RootVisualElement => _document.rootVisualElement;

    protected override void OnInitialize(TestConfiguration? testConfiguration)
    {
      _document.rootVisualElement.Clear();
      AddChild("InfoZoomContainer", out _infoZoom);
      AddChild("ScreenOverlay", out _screenOverlay);
    }

    public bool IsAnyPanelOpen()
    {
      return false;
    }

    public bool IsPointerOverScreenElement()
    {
      return EventSystem.current.IsPointerOverGameObject();
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
        return new HashSet<GameContext> {
          GameContext.Browser,
          GameContext.Hand,
          GameContext.PrimaryActionButton
        };
      }

      return null;
    }

    public float ScreenPxToElementPx(float value)
    {
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

    public void RenderScreenOverlay(FlexNode? node)
    {
      CurrentScreenOverlayNode = node;
      Reconcile(ref _screenOverlay, node ?? new FlexNode());
    }

    public void RenderInfoZoom(FlexNode node)
    {
      Reconcile(ref _infoZoom, node);
    }

    public void ClearInfoZoom()
    {
      Reconcile(ref _infoZoom, new FlexNode());
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
          left: safeLeftTop.x);
    }

    void AddChild(string elementName, out IMasonElement element)
    {
      var node = Mason.Row(elementName, new FlexStyle
      {
        Position = FlexPosition.Absolute,
        Inset = new FlexInsets()
        {
          Bottom = Mason.Px(0),
          Left = Mason.Px(0),
          Right = Mason.Px(0),
          Top = Mason.Px(0)
        },
        PickingMode = FlexPickingMode.Ignore
      });
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
