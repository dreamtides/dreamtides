#nullable enable

using NUnit.Framework;

namespace Abu.Tests
{
    public class BusyTokenTests
    {
        [SetUp]
        public void SetUp()
        {
            // Ensure no leftover tokens from other tests
            while (BusyToken.IsAnyActive)
            {
                new BusyToken().Dispose();
            }
        }

        [Test]
        public void SingleTokenTogglesIsAnyActive()
        {
            Assert.IsFalse(BusyToken.IsAnyActive);
            var token = new BusyToken();
            Assert.IsTrue(BusyToken.IsAnyActive);
            token.Dispose();
            Assert.IsFalse(BusyToken.IsAnyActive);
        }

        [Test]
        public void MultipleTokensRefCount()
        {
            var token1 = new BusyToken();
            var token2 = new BusyToken();
            Assert.IsTrue(BusyToken.IsAnyActive);

            token1.Dispose();
            Assert.IsTrue(BusyToken.IsAnyActive);

            token2.Dispose();
            Assert.IsFalse(BusyToken.IsAnyActive);
        }

        [Test]
        public void DoubleDisposeIsSafe()
        {
            var token = new BusyToken();
            token.Dispose();
            token.Dispose();
            Assert.IsFalse(BusyToken.IsAnyActive);
        }
    }
}
