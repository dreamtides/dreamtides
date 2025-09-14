#nullable enable

namespace Dreamtides.Layout
{
  using Dreamtides.Utils;
  using UnityEngine;

  public class DreamscapeLayout : MonoBehaviour
  {
    [SerializeField] ObjectLayout _draftPickLayout = null!;
    public ObjectLayout DraftPickLayout => Check(_draftPickLayout);

    T Check<T>(T? value) where T : Object =>
        Errors.CheckNotNull(value, $"{typeof(T).Name} not initialized");
  }
}
