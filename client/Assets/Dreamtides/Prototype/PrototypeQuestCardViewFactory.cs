#nullable enable

using Dreamtides.Schema;

public static class PrototypeQuestCardViewFactory
{
  public static CardView CloneCardViewWithPosition(
    CardView source,
    Position position,
    int sortingKey
  ) =>
    new CardView
    {
      Backless = source.Backless,
      CardFacing = source.CardFacing,
      CreatePosition = source.CreatePosition,
      CreateSound = source.CreateSound,
      DestroyPosition = source.DestroyPosition,
      Id = source.Id,
      Position = new ObjectPosition { Position = position, SortingKey = sortingKey },
      Prefab = source.Prefab,
      Revealed = source.Revealed,
      RevealedToOpponents = source.RevealedToOpponents,
    };

  public static CardView CloneCardViewWithPositionHidden(
    CardView source,
    Position position,
    int sortingKey
  ) =>
    new CardView
    {
      Backless = source.Backless,
      CardFacing = CardFacing.FaceDown,
      CreatePosition = source.CreatePosition,
      CreateSound = source.CreateSound,
      DestroyPosition = source.DestroyPosition,
      Id = source.Id,
      Position = new ObjectPosition { Position = position, SortingKey = sortingKey },
      Prefab = source.Prefab,
      Revealed = null,
      RevealedToOpponents = false,
    };
}
