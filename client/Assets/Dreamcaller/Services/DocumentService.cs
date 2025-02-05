#nullable enable

using Dreamcaller.Schema;
using UnityEngine;
using UnityEngine.UIElements;

namespace Dreamcaller.Services
{
  public class DocumentService : Service
  {
    [SerializeField] UIDocument _document = null!;

    public VisualElement RootVisualElement => _document.rootVisualElement;

    public bool IsAnyPanelOpen()
    {
      return false;
    }

    public bool MouseOverScreenElement()
    {
      return false;
    }

    public float ScreenPxToElementPx(float value) => value * _document.panelSettings.referenceDpi / Screen.dpi;

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
  }
}
