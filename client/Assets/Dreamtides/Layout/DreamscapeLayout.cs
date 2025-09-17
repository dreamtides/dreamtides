#nullable enable

namespace Dreamtides.Layout
{
  using Dreamtides.Utils;
  using UnityEngine;

  public class DreamscapeLayout : MonoBehaviour
  {
    [SerializeField] ObjectLayout _questDeck = null!;
    public ObjectLayout QuestDeck => Check(_questDeck);

    [SerializeField] DraftPickObjectLayout _draftPickLayout = null!;
    public DraftPickObjectLayout DraftPickLayout => Check(_draftPickLayout);

    T Check<T>(T? value) where T : Object =>
        Errors.CheckNotNull(value, $"{typeof(T).Name} not initialized");
  }
}
