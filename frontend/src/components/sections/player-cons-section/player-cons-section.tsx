import { useApp, useGame } from 'app/context';
import { useGameMessage } from 'app/hooks/use-game';
import { PlayerDomino } from '../../common/player-domino';
import { DominoTileType } from 'app/types/game';
import { getTileId } from 'app/utils';

export const PlayerConsSection = () => {
  const { setIsPending, isPending, setOpenEmptyPopup } = useApp();
  const { game, gameWasm: wasm, setSelectedDomino, selectedDomino, setPlayerChoice, playerChoice } = useGame();
  const handleMessage = useGameMessage();

  const onSuccess = () => setIsPending(false);
  const onError = () => setIsPending(false);

  const onSelect = ([i, tile]: [number, DominoTileType]) => {
    if (selectedDomino) {
      selectedDomino[0] !== i ? setSelectedDomino([i, tile]) : setSelectedDomino(undefined);
    } else {
      setSelectedDomino([i, tile]);
    }

    if (game) {
      if (playerChoice) {
        playerChoice.tile !== tile
          ? setPlayerChoice({ ...playerChoice, tile, tile_id: getTileId(tile, game.tiles) })
          : setPlayerChoice({
              ...playerChoice,
              tile: undefined,
              tile_id: undefined,
            });
      } else {
        setPlayerChoice({ tile, tile_id: getTileId(tile, game.tiles) });
      }
    }
  };

  const onTurn = () => {
    if (playerChoice?.track_id && playerChoice.tile_id) {
      const { tile_id, track_id, remove_train } = playerChoice;

      if (track_id && tile_id) {
        setIsPending((prev) => !prev);
        handleMessage({ Place: { tile_id, track_id, remove_train } }, { onSuccess, onError });
      }
    } else {
      setOpenEmptyPopup(true);
    }
  };

  const onPass = () => {
    setIsPending((prev) => !prev);
    handleMessage({ Skip: null }, { onSuccess, onError });
  };

  return (
    <div className="flex justify-between bg-[#D6FE51] py-3 px-7 rounded-2xl border border-dark-500 border-opacity-15">
      <div className="flex flex-wrap items-center gap-2 min-h-[72px]">
        {wasm &&
          wasm.playersTiles[wasm.currentPlayer - 1].map((tile, i) => (
            <PlayerDomino
              tile={tile}
              key={i}
              onClick={() => onSelect([i, tile])}
              isSelected={selectedDomino && i === selectedDomino[0]}
            />
          ))}
      </div>
      <div className="py-1 border-l border-primary pl-6 flex flex-col gap-3 min-w-[175px]">
        <button className="btn btn--primary text-dark-500 py-1.5" onClick={onTurn} disabled={isPending}>
          Turn
        </button>
        <button className="btn btn--black my-auto py-1.5" onClick={onPass} disabled={isPending}>
          Pass
        </button>
      </div>
    </div>
  );
};
