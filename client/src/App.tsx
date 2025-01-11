import "./App.css";
import { Button } from "@nextui-org/react";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faBars, faBug } from "@fortawesome/free-solid-svg-icons";

function App() {
  return (
    <main className="container">
      <h1>Welcome to Tauri + React</h1>
      <Button
        isIconOnly
        aria-label="Like"
        color="primary"
        variant="bordered"
        size="lg"
      >
        <FontAwesomeIcon icon={faBars} />
      </Button>
      <Button
        isIconOnly
        aria-label="Like"
        color="primary"
        variant="bordered"
        size="lg"
      >
        <FontAwesomeIcon icon={faBug} />
      </Button>
    </main>
  );
}

export default App;
