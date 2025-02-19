#nullable enable

using System.Collections;
using DG.Tweening;
using Dreamcaller.Components;
using Dreamcaller.Layout;
using Dreamcaller.Schema;
using Dreamcaller.Utils;
using UnityEngine;

namespace Dreamcaller.Services
{
  public class EffectService : Service
  {
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
        yield return TweenUtils.Sequence("EnlargeBeforeFiring")
          .Insert(0, source.transform.DORotate(new Vector3(280, 0, 0), 0.2f))
          .Insert(0,
            source.transform.DOMove(
              Vector3.MoveTowards(source.transform.position, Registry.MainCamera.transform.position, 20f), 0.2f))
          .WaitForCompletion();
      }

      var projectile = Registry.AssetPoolService.Create(
        Registry.AssetService.GetProjectilePrefab(command.Projectile), projectileSourcePosition.position);

      var startPosition = source.transform.position;
      var throwSequence = TweenUtils.Sequence("ProjectileThrow")
        .Insert(0, source.transform.DOMove(Vector3.Lerp(startPosition, target.transform.position, 0.1f), 0.1f))
        .Insert(0.1f, source.transform.DOMove(startPosition, 0.1f));

      if (source.GameContext.IsBattlefieldContext())
      {
        throwSequence
          .Insert(0.8f, source.transform.DOMove(originalPosition, 0.1f))
          .Insert(0.8f, source.transform.DORotate(originalRotation, 0.1f));
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
  }
}