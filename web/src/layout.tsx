import * as React from 'react'
import * as Flex from 'flexlayout-react'

import { VirtualMachine, Player } from './virtual_machine'
import { Help } from './help'
import { VM } from './vm'
import { Editor } from './editor'

const enum PaneComponent {
  Editor = 'editor',
  VM = 'vm',
  Help = 'help'
}

interface ICorewarLayoutProps {
  vm: VirtualMachine
}

export class CorewarLayout extends React.Component<ICorewarLayoutProps> {
  layoutRef = React.createRef<Flex.Layout>()

  factory(node: Flex.TabNode) {
    const vm = this.props.vm

    const component = node.getComponent()
    const config = node.getConfig()

    switch (component) {
      case PaneComponent.Editor:
        const player = this.getPlayer(config.playerId)
        config.playerId = player.id
        return (
          <Editor
            onCodeChanged={code => {
              player.champion = code
              vm.compile()
            }}
            onClosed={() => vm.removePlayer(player.id)}
          />
        )
      case PaneComponent.VM:
        return (
          <VM
            vm={vm}
            onNewPlayerRequested={() => this.onNewPlayerRequested()}
            onHelpRequested={() => this.onHelpRequested()}
          />
        )
      case PaneComponent.Help:
        return <Help />
      default:
        return null
    }
  }

  getPlayer(id: number | undefined): Player {
    let found = this.props.vm.playersById.get(id || 0)

    if (found !== undefined) return found

    return this.props.vm.newPlayer()
  }

  onModelChange(model: Flex.Model) {
    console.log(model.toJson())
    // localStorage.setItem('corewar-layout', JSON.stringify(model.toJson()))
  }

  onNewPlayerRequested() {
    const layout = this.layoutRef.current
    if (layout) {
      layout.addTabWithDragAndDropIndirect(
        'Add panel<br>(Drag to location)',
        {
          component: PaneComponent.Editor,
          name: `Champion`,
          config: {}
        },
        () => {}
      )
    }
  }

  onHelpRequested() {
    const layout = this.layoutRef.current
    if (layout) {
      layout.addTabWithDragAndDropIndirect(
        'Add panel<br>(Drag to location)',
        {
          component: PaneComponent.Help,
          name: `Documentation`
        },
        () => {}
      )
    }
  }

  render() {
    const model = Flex.Model.fromJson(DEFAULT_LAYOUT)
    return (
      <Flex.Layout
        ref={this.layoutRef}
        model={model}
        factory={this.factory.bind(this)}
        onModelChange={this.onModelChange.bind(this)}
      />
    )
  }
}

const DEFAULT_LAYOUT = {
  global: {},
  layout: {
    type: 'row',
    weight: 100,
    children: [
      {
        type: 'row',
        weight: 20,
        children: [
          {
            type: 'tabset',
            weight: 50,
            selected: 0,
            children: [
              {
                type: 'tab',
                name: 'Champion',
                component: PaneComponent.Editor,
                config: {}
              }
            ]
          },
          {
            type: 'tabset',
            weight: 50,
            selected: 0,
            location: 'bottom',
            children: [
              {
                type: 'tab',
                name: 'Champion',
                component: PaneComponent.Editor,
                config: {}
              }
            ]
          }
        ]
      },
      {
        type: 'tabset',
        weight: 80,
        selected: 0,
        children: [
          {
            type: 'tab',
            name: 'Virtual Machine',
            enableClose: false,
            component: PaneComponent.VM,
            config: {}
          }
        ]
      }
    ]
  }
}