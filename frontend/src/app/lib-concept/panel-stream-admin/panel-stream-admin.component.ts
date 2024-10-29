import {
  ChangeDetectionStrategy, Component, EventEmitter, Input, OnChanges, OnInit, Output, SimpleChanges, ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatButtonModule } from '@angular/material/button';
import { MatTooltipModule } from '@angular/material/tooltip';
import { TranslateModule, TranslateService } from '@ngx-translate/core';

import { DateTimeTimerComponent } from 'src/app/components/date-time-timer/date-time-timer.component';
import { DialogService } from 'src/app/lib-dialog/dialog.service';
import { StreamState } from 'src/app/lib-stream/stream-api.interface';

@Component({
  selector: 'app-panel-stream-admin',
  exportAs: 'appPanelStreamAdmin',
  standalone: true,
  imports: [CommonModule, MatButtonModule, MatTooltipModule, TranslateModule, 
    DateTimeTimerComponent
  ],
  templateUrl: './panel-stream-admin.component.html',
  styleUrls: ['./panel-stream-admin.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelStreamAdminComponent implements OnChanges, OnInit {
  @Input()
  public countOfViewer: number | null = null;
  @Input()
  public streamState: StreamState | null | undefined;
  @Input()
  public streamTitle: string | null | undefined;

  @Output()
  readonly changeState: EventEmitter<StreamState> = new EventEmitter();

  public statePaused = StreamState.paused;
  public statePreparing = StreamState.preparing;
  public stateStarted = StreamState.started;

  constructor(
    private translateService: TranslateService,
    private dialogService: DialogService,
  ) {
  }

  ngOnChanges(changes: SimpleChanges): void {
    // if (!!changes.streamDto && !!this.streamDTO?.credentials && this.streamDTO.state === StreamState.preparing) {
    //   this.doSettings(this.streamDTO.credentials);
    // }
  }

  ngOnInit(): void {
  }

  // ** Public API **

  public isBntPrepare(streamState: StreamState | null | undefined): boolean {
    const streamState2: StreamState = (streamState || StreamState.waiting);
    console.log(`isBntPrepare(${streamState})=`, [StreamState.waiting, StreamState.stopped].includes(streamState2)); // #
    return [StreamState.waiting, StreamState.stopped].includes(streamState2);
  }
  public isBntStart(streamState: StreamState | null | undefined): boolean {
    const streamSate: StreamState = (streamState || StreamState.waiting);
    return [StreamState.preparing, StreamState.paused].includes(streamSate);
  }
  public isBntPause(streamState: StreamState | null | undefined): boolean {
    const streamSate: StreamState = (streamState || StreamState.waiting);
    return [StreamState.started].includes(streamSate);
  }
  public isBtnStop(streamState: StreamState | null | undefined): boolean {
    const streamSate: StreamState = (streamState || StreamState.waiting);
    return [StreamState.preparing, StreamState.started, StreamState.paused].includes(streamSate);
  }

  /*public doSettings(credentials: Credentials | null): void {
    if (!credentials || !credentials.rtmpPublishPath) { return; }
    const dataParams: SettingsData = {
      rtmpPath: credentials.rtmpPublishPath,
      rtmpStreamName: credentials.rtmpPublishStreamName
    };
    this.dialogService.openComponent(SettingsComponent, dataParams);
  }*/
  public doChangeState(newStreamState: StreamState): void {
    if (this.streamState != null) {
      this.changeState.emit(newStreamState);
    }
  }
  public doChangeOnStateStopped(): void {
    if (this.streamState != null) {
      const message = this.translateService.instant('panel-stream-admin.sure_you_want_stop_stream', { title: this.streamTitle });
      const params = { btnNameCancel: 'buttons.no', btnNameAccept: 'buttons.yes' };
      this.dialogService.openConfirmation(message, '', params)
        .then((response) => {
          if (!!response) {
            this.doChangeState(StreamState.stopped);
          }
        });
    }
  }

  // ** Private API **

}
