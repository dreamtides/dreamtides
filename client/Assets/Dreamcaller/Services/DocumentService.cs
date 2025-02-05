#nullable enable

using Dreamcaller.Masonry;
using Dreamcaller.Schema;
using UnityEngine;
using UnityEngine.UIElements;

namespace Dreamcaller.Services
{
  public class DocumentService : Service
  {
    [SerializeField] UIDocument _document = null!;
    IMasonElement _infoZoom = null!;

    public VisualElement RootVisualElement => _document.rootVisualElement;

    protected override void OnInitialize()
    {
      _document.rootVisualElement.Clear();
      AddChild("InfoZoomContainer", out _infoZoom);
    }

    public bool IsAnyPanelOpen()
    {
      return false;
    }

    public bool MouseOverScreenElement()
    {
      return false;
    }

    public float ScreenPxToElementPx(float value) => value * _document.panelSettings.referenceDpi / Screen.dpi;

    public void ClearInfoZoom()
    {
      Reconcile(ref _infoZoom, new FlexNode());
    }

    public void RenderInfoZoom(FlexNode node)
    {
      Reconcile(ref _infoZoom, node);
    }

    public DimensionGroup GetSafeArea()
    {
      var panel = RootVisualElement.panel;
      var safeLeftTop = RuntimePanelUtils.ScreenToPanel(
        panel,
        new Vector2(Screen.safeArea.xMin, Screen.height - Screen.safeArea.yMax)
      );
      var safeRightBottom = RuntimePanelUtils.ScreenToPanel(
        panel,
        new Vector2(Screen.width - Screen.safeArea.xMax, Screen.safeArea.yMin)
      );

      return Mason.GroupPx(safeLeftTop.x, safeLeftTop.y, safeRightBottom.x, safeRightBottom.y);
    }

    void AddChild(string elementName, out IMasonElement element)
    {
      var node = Mason.Row(elementName, new FlexStyle
      {
        Position = FlexPosition.Absolute,
        Inset = Mason.AllPx(0),
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
