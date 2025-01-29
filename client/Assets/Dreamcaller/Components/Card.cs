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
    [SerializeField] RectTransform _backingQuad = null!;
    Registry _registry = null!;
    CardView _cardView = null!;

    public CardView CardView => Errors.CheckNotNull(_cardView);

    public void Render(Registry registry, CardView view, Sequence sequence)
    {
      _registry = registry;
      _cardView = view;
      _name.text = view.Revealed?.Name;
    }

    public void TurnFaceDown(Sequence sequence)
    {
    }

    protected override void OnSetGameContext(GameContext oldContext, GameContext newContext)
    {
      if (newContext.IsBattlefieldContext())
      {
        _backingQuad.localScale = new Vector3(2.5f, 3f, 1);
      }
      else
      {
        _backingQuad.localScale = new Vector3(2.5f, 4f, 1);
      }
    }
  }
}
