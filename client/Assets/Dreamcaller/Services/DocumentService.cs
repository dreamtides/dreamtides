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
      AddChild("InfoZoom", out _infoZoom);
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

      return new DimensionGroup
      {
        Left = new Dimension { Value = safeLeftTop.x, Unit = DimensionUnit.Pixels },
        Top = new Dimension { Value = safeLeftTop.y, Unit = DimensionUnit.Pixels },
        Right = new Dimension { Value = safeRightBottom.x, Unit = DimensionUnit.Pixels },
        Bottom = new Dimension { Value = safeRightBottom.y, Unit = DimensionUnit.Pixels }
      };
    }

    void AddChild(string elementName, out IMasonElement element)
    {
      var node = MasonUtils.Row(elementName, new FlexStyle
      {
        Position = FlexPosition.Absolute,
        Inset = MasonUtils.AllDip(0),
        PickingMode = FlexPickingMode.Ignore
      });
      element = Mason.Render(Registry, node);
      _document.rootVisualElement.Add(element.Self);
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
