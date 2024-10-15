import {
  ChangeDetectionStrategy, Component, ElementRef, Input, OnChanges, OnInit, Renderer2, SimpleChanges, ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { TranslateService } from '@ngx-translate/core';
import { HtmlElemUtil } from 'src/app/utils/html-elem.util';

export enum StreamStatus {
  Waiting = 'Waiting',
  Preparing = 'Preparing',
  Started = 'Started',
  Stopped = 'Stopped',
  Paused = 'Paused'
}

const ATTR_STATE = 'state';

@Component({
  selector: 'app-panel-stream-state',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './panel-stream-state.component.html',
  styleUrls: ['./panel-stream-state.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelStreamStateComponent implements OnChanges, OnInit {
  @Input()
  public streamState: string | null | undefined = null;
  @Input()
  public theme: string | null = null;

  public state: string | null = null;
  public valueText: string | null = null;
  
  private innStreamStatus: StreamStatus = StreamStatus.Waiting;

  constructor(
    private renderer: Renderer2,
    public hostRef: ElementRef<HTMLElement>,
    private translateService: TranslateService,
  ) {
  }

  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['streamState']) {
      this.innStreamStatus = this.createStreamStatus(this.streamState);
      HtmlElemUtil.setAttr(this.renderer, this.hostRef, ATTR_STATE, this.innStreamStatus);
      this.state = this.innStreamStatus;
      this.valueText = this.getValueText(this.innStreamStatus);
    }
  }

  ngOnInit(): void {
  }

  // ** Private API **

  private createStreamStatus(value: string | null | undefined): StreamStatus {
    let result: StreamStatus = StreamStatus.Waiting;
    switch (value) {
      case StreamStatus.Waiting: result = StreamStatus.Waiting; break;
      case StreamStatus.Preparing: result = StreamStatus.Preparing; break;
      case StreamStatus.Started: result = StreamStatus.Started; break;
      case StreamStatus.Stopped: result = StreamStatus.Stopped; break;
      case StreamStatus.Paused: result = StreamStatus.Paused; break;
    }
    return result;
  }
  private isActive(streamStatus: StreamStatus): boolean {
    return [StreamStatus.Preparing, StreamStatus.Started, StreamStatus.Paused].includes(streamStatus);
  }
  private getValueText(streamState: StreamStatus): string {
    let res = '';
    switch (streamState) {
      case StreamStatus.Waiting: res = 'stream_state.has_not_started'; break;
      case StreamStatus.Preparing: res = 'stream_state.has_not_started'; break;
      case StreamStatus.Paused: res = 'stream_state.has_paused'; break;
      case StreamStatus.Stopped: res = 'stream_state.has_ended'; break;
    }
    return (!!res ? this.translateService.instant(res): '');
  }
}
