using System.Collections;
using NUnit.Framework;
using UnityEngine.TestTools;
using Dreamtides.Services;
using Dreamtides.Utils;
using Dreamtides.Components;
using Dreamtides.UnityInternal;
using UnityEngine;
using System.Linq;

namespace Dreamtides.Tests
{
  public class BasicActionsTest
  {
    [TearDown]
    public void TearDown()
    {
      Registry.TestConfiguration = null;
    }

    [UnityTest]
    public IEnumerator TestBasicLayout()
    {

    }
  }
}
