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
    [SerializeField] TextMeshPro _rulesText = null!;
    [SerializeField] Transform _cardFrame = null!;
    Registry _registry = null!;
    CardView _cardView = null!;

    public CardView CardView => Errors.CheckNotNull(_cardView);

    public void Render(Registry registry, CardView view, Sequence sequence)
    {
      _registry = registry;
      _cardView = view;
      _name.text = view.Revealed?.Name;
      _rulesText.text = view.Revealed?.RulesText;
    }

    public void TurnFaceDown(Sequence sequence)
    {
    }

    protected override void OnSetGameContext(GameContext oldContext, GameContext newContext)
    {
      if (newContext.IsBattlefieldContext())
      {
        _cardFrame.gameObject.SetActive(false);
        _name.gameObject.SetActive(false);
        _rulesText.gameObject.SetActive(false);
      }
      else
      {
        _cardFrame.gameObject.SetActive(true);
        _name.gameObject.SetActive(true);
        _rulesText.gameObject.SetActive(true);
      }
    }
  }
}
