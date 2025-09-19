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
    DraftPickObjectLayout _draftPickLayout = null!;
    public DraftPickObjectLayout DraftPickLayout => Check(_draftPickLayout);

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
    BrowserBackground _dreamscapeBackgroundOverlay = null!;
    public BrowserBackground DreamscapeBackgroundOverlay => Check(_dreamscapeBackgroundOverlay);

    T Check<T>(T? value)
      where T : Object => Errors.CheckNotNull(value, $"{typeof(T).Name} not initialized");
  }
}
