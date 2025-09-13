#nullable enable

using System.Collections.Generic;
using System.Runtime.CompilerServices;
using Dreamtides.Components;
using Dreamtides.Schema;
using Dreamtides.Utils;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]
namespace Dreamtides.Services
{
  public class ArrowService : Service
  {
    [SerializeField] Arrow _redArrowPrefab = null!;
    [SerializeField] Arrow _greenArrowPrefab = null!;
    [SerializeField] Arrow _blueArrowPrefab = null!;
    internal readonly List<Arrow> _arrows = new();

    public void HandleDisplayArrows(List<DisplayArrow> arrows)
    {
      HideArrows();
      var arrowIndex = 0;
      foreach (var arrow in arrows)
      {
        var arrowInstance = ComponentUtils.Instantiate(ArrowForType(arrow.Color));
        var source = Registry.CardService.GetGameObject(arrow.Source);
        var target = Registry.CardService.GetGameObject(arrow.Target);
        if (source.SortingGroup)
        {
          arrowInstance.SortingGroup.sortingLayerID = source.SortingGroup.sortingLayerID;
          arrowInstance.SortingGroup.sortingOrder = source.SortingGroup.sortingOrder + 1;
        }
        _arrows.Add(arrowInstance);
        arrowInstance.Source = source.transform;
        arrowInstance.Target = target.transform;

        var yOffset = arrowIndex * -0.2f;
        arrowInstance.SourceOffset = new Vector3(0, 0, yOffset);
        arrowInstance.TargetOffset = new Vector3(0, 0, yOffset);

        arrowIndex++;
      }
    }

    public void HideArrows()
    {
      foreach (var arrow in _arrows)
      {
        Destroy(arrow.gameObject);
      }

      _arrows.Clear();
    }

    Arrow ArrowForType(ArrowStyle style) => style switch
    {
      ArrowStyle.Red => _redArrowPrefab,
      ArrowStyle.Green => _greenArrowPrefab,
      ArrowStyle.Blue => _blueArrowPrefab,
      _ => throw Errors.UnknownEnumValue(style)
    };
  }
}