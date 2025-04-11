#nullable enable

using System.Collections;
using System.Collections.Generic;
using Dreamtides.Layout;
using UnityEngine;

namespace Dreamtides.Services
{
  public class TestHelperService : Service
  {
    public IEnumerator WaitForIdle()
    {
      var epsilon = 0.1f;
      var waitTime = 1.0f;
      var lastPositions = new Dictionary<Displayable, Vector3>();
      var timer = 0.0f;

      var displayables = FindObjectsByType<Displayable>(FindObjectsSortMode.None);
      foreach (var displayable in displayables)
      {
        lastPositions[displayable] = displayable.transform.position;
      }

      yield return new WaitForEndOfFrame();

      // Keep checking until stable for waitTime seconds
      while (timer < waitTime)
      {
        var hasMovement = false;
        displayables = FindObjectsByType<Displayable>(FindObjectsSortMode.None);

        // Check for movement or new objects
        foreach (var displayable in displayables)
        {
          var currentPosition = displayable.transform.position;

          if (lastPositions.TryGetValue(displayable, out var lastPosition))
          {
            if (Vector3.Distance(currentPosition, lastPosition) > epsilon)
            {
              // Movement detected
              hasMovement = true;
              lastPositions[displayable] = currentPosition;
            }
          }
          else
          {
            // New object found
            lastPositions[displayable] = currentPosition;
            hasMovement = true;
          }
        }

        // Reset or increment timer based on movement
        if (hasMovement)
        {
          timer = 0.0f;
        }
        else
        {
          timer += Time.deltaTime;
        }

        yield return new WaitForEndOfFrame();
      }
    }
  }
}
