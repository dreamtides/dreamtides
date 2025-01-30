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
    [SerializeField] MeshRenderer _cardImage = null!;
    [SerializeField] Material _material1 = null!;
    [SerializeField] Material _material2 = null!;
    [SerializeField] Material _material3 = null!;
    [SerializeField] Material _material4 = null!;
    [SerializeField] Material _material5 = null!;

    Registry _registry = null!;
    CardView _cardView = null!;

    public CardView CardView => Errors.CheckNotNull(_cardView);

    public void Render(Registry registry, CardView view, Sequence sequence)
    {
      gameObject.name = view.Revealed?.Name ?? "Hidden Card";
      _registry = registry;
      _cardView = view;
      _name.text = view.Revealed?.Name;
      _rulesText.text = view.Revealed?.RulesText;

      if (view.Revealed?.Image?.Image.Contains("1633431262") == true)
      {
        _cardImage.material = _material1;
      }
      else if (view.Revealed?.Image?.Image.Contains("2027158310") == true)
      {
        _cardImage.material = _material2;
      }
      else if (view.Revealed?.Image?.Image.Contains("2269064809") == true)
      {
        _cardImage.material = _material3;
      }
      else if (view.Revealed?.Image?.Image.Contains("2269064817") == true)
      {
        _cardImage.material = _material4;
      }
      else if (view.Revealed?.Image?.Image.Contains("2521694543") == true)
      {
        _cardImage.material = _material5;
      }
    }

    public void TurnFaceDown(Sequence sequence)
    {
    }

    public override bool CanHandleMouseEvents() => true;

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
