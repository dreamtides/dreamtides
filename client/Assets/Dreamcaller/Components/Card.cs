#nullable enable

using DG.Tweening;
using Dreamcaller.Layout;
using Dreamcaller.Schema;
using Dreamcaller.Services;
using Dreamcaller.Utils;
using TMPro;
using UnityEngine;

namespace Dreamcaller.Components
{
  public class Card : Displayable
  {
    [SerializeField] TextMeshPro _name = null!;
    Registry _registry = null!;
    CardView _cardView = null!;

    public CardView CardView => Errors.CheckNotNull(_cardView);

    public void Render(Registry registry, CardView view, Sequence sequence)
    {
      _registry = registry;
      _cardView = view;
    }

    public void TurnFaceDown(Sequence sequence)
    {
    }
  }
}
