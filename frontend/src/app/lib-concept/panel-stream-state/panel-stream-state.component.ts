import {
  ChangeDetectionStrategy, Component, ElementRef, Input, OnChanges, OnInit, Renderer2, SimpleChanges, ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { TranslateService } from '@ngx-translate/core';

import { StreamState } from 'src/app/lib-stream/stream-api.interface';
import { HtmlElemUtil } from 'src/app/utils/html-elem.util';

const ATTR_STATE = 'state';

@Component({
  selector: 'app-panel-stream-state',
  exportAs: 'appPanelStreamState',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './panel-stream-state.component.html',
  styleUrl: './panel-stream-state.component.scss',
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelStreamStateComponent implements OnChanges, OnInit {
  @Input()
  public streamState: StreamState | null | undefined = null;

  public strmStWaiting: StreamState = StreamState.waiting;
  public strmStPreparing: StreamState = StreamState.preparing;
  public strmStStarted: StreamState = StreamState.started;
  public strmStPaused: StreamState = StreamState.paused;
  public strmStStopped: StreamState = StreamState.stopped;

  public valueText: string | null = null;
  
  constructor(
    private renderer: Renderer2,
    public hostRef: ElementRef<HTMLElement>,
    private translateService: TranslateService,
  ) {
  }

  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['streamState']) {
      const streamState = this.streamState || StreamState.waiting;
      HtmlElemUtil.setAttr(this.renderer, this.hostRef, ATTR_STATE, streamState);
      this.valueText = this.getValueText(streamState);
    }
  }

  ngOnInit(): void {
  }

  // ** Private API **

  private isActive(streamStatus: StreamState): boolean {
    return [StreamState.preparing, StreamState.started, StreamState.paused].includes(streamStatus);
  }
  private getValueText(streamState: StreamState): string {
    let res = '';
    switch (streamState) {
      case StreamState.waiting: res = 'stream_state.has_not_started'; break;
      case StreamState.preparing: res = 'stream_state.has_not_started'; break;
      case StreamState.paused: res = 'stream_state.has_paused'; break;
      case StreamState.stopped: res = 'stream_state.has_ended'; break;
    }
    return (!!res ? this.translateService.instant(res): '');
  }
}
