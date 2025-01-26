#nullable enable

using DG.Tweening;
using Dreamcaller.Layout;
using Dreamcaller.Schema;
using Dreamcaller.Utils;

namespace Dreamcaller.Components
{
  public class Card : Displayable
  {
    CardView? _cardView;
    public CardView CardView { get => Errors.CheckNotNull(_cardView); }

    public void Render(CardView view, Sequence sequence)
    {
    }

    public void TurnFaceDown(Sequence sequence)
    {
    }
  }
}
