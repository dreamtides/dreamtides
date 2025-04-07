#nullable enable

using Dreamcaller.Schema;
using TMPro;
using UnityEngine;
using System.Collections;

namespace Dreamcaller.Components
{
  public class EnemyMessage : MonoBehaviour
  {
    [SerializeField] SpriteRenderer _background = null!;
    [SerializeField] TextMeshPro _messageText = null!;
    [SerializeField] float _fadeInDuration = 0.3f;
    [SerializeField] float _fadeOutDuration = 0.3f;

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