#nullable enable

namespace Dreamtides.Layout
{
  using Dreamtides.Utils;
  using UnityEngine;

  public class DreamscapeLayout : MonoBehaviour
  {
    [SerializeField]
    ObjectLayout _questDeck = null!;
    public ObjectLayout QuestDeck => Check(_questDeck);

    [SerializeField]
    SitePickObjectLayout _draftPickLayout = null!;
    public SitePickObjectLayout DraftPickLayout => Check(_draftPickLayout);

    [SerializeField]
    ObjectLayout _destroyedQuestCards = null!;
    public ObjectLayout DestroyedQuestCards => Check(_destroyedQuestCards);

    [SerializeField]
    Transform _aboveQuestDeck = null!;
    public Transform AboveQuestDeck => Check(_aboveQuestDeck);

    [SerializeField]
    StandardObjectLayout _shopLayout = null!;
    public StandardObjectLayout ShopLayout => Check(_shopLayout);

    [SerializeField]
    StandardObjectLayout _dreamsignDisplay = null!;
    public StandardObjectLayout DreamsignDisplay => Check(_dreamsignDisplay);

    [SerializeField]
    StandardObjectLayout _journeyChoiceDisplay = null!;
    public StandardObjectLayout JourneyChoiceDisplay => Check(_journeyChoiceDisplay);

    [SerializeField]
    TemptingOfferObjectLayout _temptingOfferDisplay = null!;
    public TemptingOfferObjectLayout TemptingOfferDisplay => Check(_temptingOfferDisplay);

    [SerializeField]
    ObjectLayout _questEffectPosition = null!;
    public ObjectLayout QuestEffectPosition => Check(_questEffectPosition);

    T Check<T>(T? value)
      where T : Object => Errors.CheckNotNull(value, $"{typeof(T).Name} not initialized");
  }
}
