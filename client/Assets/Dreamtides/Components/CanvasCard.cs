#nullable enable

using System.Runtime.CompilerServices;
using TMPro;
using UnityEngine;
using UnityEngine.UI;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Components
{
  public class CanvasCard : MonoBehaviour
  {
    [SerializeField]
    Canvas _canvas = null!;

    [SerializeField]
    RectTransform _root = null!;

    [SerializeField]
    Image _cardImage = null!;

    [SerializeField]
    Image _cardFrame = null!;

    [SerializeField]
    TextMeshProUGUI _cardName = null!;

    [SerializeField]
    TextMeshProUGUI _cardType = null!;

    [SerializeField]
    TextMeshProUGUI _rulesText = null!;

    [SerializeField]
    Image _costBackground = null!;

    [SerializeField]
    TextMeshProUGUI _costText = null!;

    [SerializeField]
    Image _sparkBackground = null!;

    [SerializeField]
    TextMeshProUGUI _sparkText = null!;
  }
}
