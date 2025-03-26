#nullable enable

using Dreamcaller.Components;
using TMPro;
using UnityEngine;

namespace Dreamcaller
{
  public class BattlefieldNumber : MonoBehaviour
  {
    [SerializeField] TextMeshPro _text = null!;
    [SerializeField] TimedEffect _onChange = null!;

    public void SetText(string text, bool animate)
    {
      if (_text.text != text)
      {
        _text.text = text;
        if (animate)
        {
          // Toggle to restart animation if needed
          _onChange.gameObject.SetActive(false);
          _onChange.gameObject.SetActive(true);
        }
      }
    }
  }
}