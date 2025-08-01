#nullable enable

using System.Collections;
using DG.Tweening;
using Dreamtides.Layout;
using Dreamtides.Schema;
using Dreamtides.Utils;
using UnityEngine;

namespace Dreamtides.Services
{
  public class EffectService : Service
  {
    /// <summary>
    /// Handles a DisplayEffectCommand by creating an effect and waiting for it to
    /// finish.
    /// </summary>
    public IEnumerator HandleDisplayEffectCommand(DisplayEffectCommand command)
    {
      var target = Registry.LayoutService.GetGameObject(command.Target);
      var effectPosition = target.DisplayEffectPosition;
      var effect = Registry.AssetService.GetEffectPrefab(command.Effect);
      // TODO: Figure out correct rotation for effects.

      if (effectPosition)
      {
        Registry.AssetPoolService.Create(effect, effectPosition.position);
        effect.transform.localEulerAngles = new Vector3(90, 0, 0);
      }
      else
      {
        Registry.AssetPoolService.Create(effect, target.transform.position);
        effect.transform.localEulerAngles = new Vector3(90, 0, 0);
      }

      if (command.Sound != null)
      {
        Registry.SoundService.Play(command.Sound);
      }

      yield return new WaitForSeconds(command.Duration.ToSeconds());
    }

    /// <summary>
    /// Handles a FireProjectileCommand by creating a projectile and animating
    /// its flight.
    /// </summary>
    public IEnumerator HandleFireProjectileCommand(FireProjectileCommand command)
    {
      var source = Registry.LayoutService.GetGameObject(command.SourceId);
      var target = Registry.LayoutService.GetGameObject(command.TargetId);
      var originalPosition = source.transform.position;
      var originalRotation = source.transform.rotation.eulerAngles;
      var projectileSourcePosition = source.ProjectileSourcePosition ?
          source.ProjectileSourcePosition : source.transform;

      if (source.GameContext.IsBattlefieldContext())
      {
        // Enlarge before firing
        // yield return TweenUtils.Sequence("EnlargeBeforeFiring")
        //   .Insert(0, source.transform.DORotate(new Vector3(280, 0, 0), 0.2f))
        //   .Insert(0,
        //     source.transform.DOMove(
        //       Vector3.MoveTowards(source.transform.position, Registry.Layout.MainCamera.transform.position, 20f), 0.2f))
        //   .WaitForCompletion();
      }

      var projectile = Registry.AssetPoolService.Create(
        Registry.AssetService.GetProjectilePrefab(command.Projectile), projectileSourcePosition.position);

      var startPosition = source.transform.position;
      var throwSequence = TweenUtils.Sequence("ProjectileThrow")
        .Insert(0, source.transform.DOMove(Vector3.Lerp(startPosition, target.transform.position, 0.1f), 0.1f))
        .Insert(0.1f, source.transform.DOMove(startPosition, 0.1f));

      if (source.GameContext.IsBattlefieldContext())
      {
        // throwSequence
        //   .Insert(0.8f, source.transform.DOMove(originalPosition, 0.1f))
        //   .Insert(0.8f, source.transform.DORotate(originalRotation, 0.1f));
      }

      yield return projectile.Fire(
        Registry,
        target.transform,
        command.TravelDuration,
        command.AdditionalHit,
        command.AdditionalHitDelay,
        command.FireSound,
        command.ImpactSound);

      if (command.HideOnHit)
      {
        target.gameObject.transform.position = Vector3.zero;
      }

      if (command.WaitDuration != null)
      {
        yield return new WaitForSeconds(command.WaitDuration.ToSeconds());
      }

      if (command.JumpToPosition != null)
      {
        var sequence = TweenUtils.Sequence("JumpToPosition");
        Registry.LayoutService.MoveObject(target, command.JumpToPosition, sequence);
        yield return sequence.WaitForCompletion();
      }

      if (throwSequence.IsActive())
      {
        yield return throwSequence.WaitForCompletion();
      }
    }

    public IEnumerator HandleDissolveCommand(DissolveCardCommand command)
    {
      var target = Registry.LayoutService.GetCard(command.Target);
      yield return target.StartDissolve(command);
    }

    public void HandleSetCardTrailCommand(SetCardTrailCommand command)
    {
      foreach (var cardId in command.CardIds)
      {
        var card = Registry.LayoutService.GetCardIfExists(cardId);
        if (card)
        {
          card.SetCardTrail(command.Trail, command.Duration.ToSeconds());
        }
      }
    }
  }
}