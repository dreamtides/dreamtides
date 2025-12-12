#nullable enable

using System.Runtime.CompilerServices;
using Dreamtides.Components;
using Dreamtides.Utils;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Layout
{
  public class DreamscapeLayout : Displayable
  {
    [SerializeField]
    internal ObjectLayout _questDeck = null!;
    public ObjectLayout QuestDeck => Check(_questDeck);

    [SerializeField]
    internal ObjectLayout _questUserIdentityCard = null!;
    public ObjectLayout QuestUserIdentityCard => Check(_questUserIdentityCard);

    [SerializeField]
    internal QuestDeckBrowserObjectLayout _questDeckBrowserPortrait = null!;

    [SerializeField]
    internal QuestDeckBrowserObjectLayout _questDeckBrowserLandscape = null!;

    public QuestDeckBrowserObjectLayout QuestDeckBrowser =>
      Registry.IsLandscape ? Check(_questDeckBrowserLandscape) : Check(_questDeckBrowserPortrait);

    [SerializeField]
    internal EssenceTotal _essenceTotalPortrait = null!;

    [SerializeField]
    internal EssenceTotal _essenceTotalLandscape = null!;

    public EssenceTotal EssenceTotal =>
      Registry.IsLandscape ? Check(_essenceTotalLandscape) : Check(_essenceTotalPortrait);

    [SerializeField]
    internal SitePickObjectLayout _draftPickLayout = null!;
    public SitePickObjectLayout DraftPickLayout => Check(_draftPickLayout);

    [SerializeField]
    internal ObjectLayout _destroyedQuestCards = null!;
    public ObjectLayout DestroyedQuestCards => Check(_destroyedQuestCards);

    [SerializeField]
    internal Transform _aboveQuestDeck = null!;
    public Transform AboveQuestDeck => Check(_aboveQuestDeck);

    [SerializeField]
    internal StandardObjectLayout _shopLayout = null!;
    public StandardObjectLayout ShopLayout => Check(_shopLayout);

    [SerializeField]
    internal DreamsignDisplayLayout _dreamsignDisplay = null!;
    public DreamsignDisplayLayout DreamsignDisplay => Check(_dreamsignDisplay);

    [SerializeField]
    internal StandardObjectLayout _journeyChoiceDisplay = null!;
    public StandardObjectLayout JourneyChoiceDisplay => Check(_journeyChoiceDisplay);

    [SerializeField]
    internal TemptingOfferObjectLayout _temptingOfferDisplay = null!;
    public TemptingOfferObjectLayout TemptingOfferDisplay => Check(_temptingOfferDisplay);

    [SerializeField]
    internal ObjectLayout _questEffectPosition = null!;
    public ObjectLayout QuestEffectPosition => Check(_questEffectPosition);

    [SerializeField]
    internal StartBattleObjectLayout _startBattleLayout = null!;
    public StartBattleObjectLayout StartBattleLayout => Check(_startBattleLayout);

    [SerializeField]
    internal Displayable _essenceTotalWorldPosition = null!;
    public Displayable EssenceTotalWorldPosition => Check(_essenceTotalWorldPosition);

    T Check<T>(T? value)
      where T : Object => Errors.CheckNotNull(value, $"{typeof(T).Name} not initialized");
  }
}
