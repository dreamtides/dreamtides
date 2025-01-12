import { Position } from "../../bindings";

type UserHandProps = { position: Position };

export default function UserHand({}: UserHandProps) {
  return (
    <div className="flex bg-blue-600" style={{ height: "30dvh" }}>
      User Hand
    </div>
  );
}
