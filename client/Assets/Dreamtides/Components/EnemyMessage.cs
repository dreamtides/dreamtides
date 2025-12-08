#nullable enable

using System.Collections;
using System.Runtime.CompilerServices;
using Dreamtides.Schema;
using TMPro;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Components
{
  public class EnemyMessage : MonoBehaviour
  {
    [SerializeField]
    internal SpriteRenderer _background = null!;

    [SerializeField]
    internal TextMeshPro _messageText = null!;

    [SerializeField]
    internal float _fadeInDuration = 0.3f;

    [SerializeField]
    internal float _fadeOutDuration = 0.3f;

    private Coroutine? _activeCoroutine;

    public void Show(DisplayEnemyMessageCommand command)
    {
      if (_activeCoroutine != null)
      {
        StopCoroutine(_activeCoroutine);
      }

      SetAlpha(0f);
      gameObject.SetActive(true);
      _messageText.text = command.Message;
      _activeCoroutine = StartCoroutine(ShowMessageCoroutine(command.ShowDuration.ToSeconds()));
    }

    private IEnumerator ShowMessageCoroutine(float showDuration)
    {
      yield return FadeCoroutine(0f, 1f, _fadeInDuration);
      yield return new WaitForSeconds(showDuration);
      yield return FadeCoroutine(1f, 0f, _fadeOutDuration);
      _activeCoroutine = null;
      gameObject.SetActive(false);
    }

    private IEnumerator FadeCoroutine(float startAlpha, float endAlpha, float duration)
    {
      float elapsedTime = 0f;

      while (elapsedTime < duration)
      {
        float alpha = Mathf.Lerp(startAlpha, endAlpha, elapsedTime / duration);
        SetAlpha(alpha);

        elapsedTime += Time.deltaTime;
        yield return null;
      }

      // Ensure we end at the exact target alpha
      SetAlpha(endAlpha);
    }

    private void SetAlpha(float alpha)
    {
      Color bgColor = _background.color;
      bgColor.a = alpha;
      _background.color = bgColor;

      Color textColor = _messageText.color;
      textColor.a = alpha;
      _messageText.color = textColor;
    }
  }
}
