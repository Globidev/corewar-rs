import * as React from 'react'
import { observer } from 'mobx-react'

import { VirtualMachine } from '../../virtual_machine'
import { PlayerInfo } from '../../corewar'
import { toCssColor, titledInfo } from './common'

interface IContendersPanelProps {
  vm: VirtualMachine
  coverages: Map<number, number>
}

@observer
export class ContendersPanel extends React.Component<IContendersPanelProps> {
  changePlayerId(currentId: number) {
    const newIdAsString = prompt('New id', currentId.toString())
    if (newIdAsString) {
      const newId = parseInt(newIdAsString)
      this.props.vm.changePlayerId(currentId, newId)
    }
  }

  render() {
    const vm = this.props.vm

    return (
      <div>
        <div>{vm.playersById.size} contenders:</div>
        {Array.from(vm.playersById.values()).map((player, i) => {
          if (vm.cycles === null) return null

          let playerInfo = vm.engine.player_info(player.id) as PlayerInfo | null
          if (playerInfo === null) return null

          let championInfo = vm.engine.champion_info(player.id)
          const coverage = this.props.coverages.get(player.id) || 0

          const playerIdInput = (
            <div onClick={() => this.changePlayerId(player.id)}>{player.id}</div>
          )

          return (
            <details key={i} style={{ color: toCssColor(player.color) }}>
              <summary>{playerInfo.champion_name()}</summary>
              {titledInfo('Player ID', playerIdInput)}
              {titledInfo('Size', playerInfo.champion_size)}
              {titledInfo('Coverage', `${((coverage / 4096) * 100).toFixed(2)} %`)}
              {titledInfo('Processes', championInfo.process_count)}
              {titledInfo('Last live', championInfo.last_live)}
            </details>
          )
        })}
      </div>
    )
  }
}
