import {
  ChangeDetectionStrategy, Component, EventEmitter, Input, OnChanges, OnInit, Output, SimpleChanges, ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatButtonModule } from '@angular/material/button';
import { TranslateModule, TranslateService } from '@ngx-translate/core';

import { DateTimeTimerComponent } from 'src/app/components/date-time-timer/date-time-timer.component';
import { DialogService } from 'src/app/lib-dialog/dialog.service';
import { StreamState } from 'src/app/lib-stream/stream-api.interface';

@Component({
  selector: 'app-panel-stream-admin',
  exportAs: 'appPanelStreamAdmin',
  standalone: true,
  imports: [CommonModule, MatButtonModule, TranslateModule, 
    DateTimeTimerComponent
  ],
  templateUrl: './panel-stream-admin.component.html',
  styleUrl: './panel-stream-admin.component.scss',
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelStreamAdminComponent implements OnChanges, OnInit {
  @Input()
  public countOfViewer: number | null = null;
  @Input()
  public state: StreamState | null | undefined;
  @Input()
  public startDate: Date | null | undefined;
  @Input()
  public title: string | null | undefined;

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

  public isShowSettings(): boolean {
    return true; // !!this.streamDto?.credentials
  }
  public isShowTimer(state: StreamState | null | undefined): boolean {
    const state2: StreamState = (state || StreamState.waiting);
    return [StreamState.preparing, StreamState.started].includes(state2);
  }
  public isShowBntPrepare(state: StreamState | null | undefined): boolean {
    const state2: StreamState = (state || StreamState.waiting);
    return [StreamState.waiting, StreamState.stopped].includes(state2);
  }
  public isShowBntStart(state: StreamState | null | undefined): boolean {
    const state2: StreamState = (state || StreamState.waiting);
    return [StreamState.preparing, StreamState.paused].includes(state2);
  }
  public isShowBntPause(state: StreamState | null | undefined): boolean {
    const state2: StreamState = (state || StreamState.waiting);
    return [StreamState.started].includes(state2);
  }
  public isBtnStop(state: StreamState | null | undefined): boolean {
    const state2: StreamState = (state || StreamState.waiting);
    return [StreamState.preparing, StreamState.started, StreamState.paused].includes(state2);
  }

  public doSettings(): void {
  }
  /*public doSettings(credentials: Credentials | null): void {
    if (!credentials || !credentials.rtmpPublishPath) { return; }
    const dataParams: SettingsData = {
      rtmpPath: credentials.rtmpPublishPath,
      rtmpStreamName: credentials.rtmpPublishStreamName
    };
    this.dialogService.openComponent(SettingsComponent, dataParams);
  }*/
  public doChangeState(newState: StreamState): void {
    if (this.state != null) {
      this.changeState.emit(newState);
    }
  }
  public doChangeOnStateStopped(): void {
    if (this.state != null) {
      const message = this.translateService.instant('panel-stream-admin.sure_you_want_stop_stream', { title: this.title });
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
