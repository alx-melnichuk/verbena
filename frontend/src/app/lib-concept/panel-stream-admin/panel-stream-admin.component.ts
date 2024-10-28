import {
  ChangeDetectionStrategy, Component, EventEmitter, Input, OnChanges, OnInit, Output, SimpleChanges, ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatButtonModule } from '@angular/material/button';
import { MatTooltipModule } from '@angular/material/tooltip';
import { TranslateModule, TranslateService } from '@ngx-translate/core';

import { DateTimeTimerComponent } from 'src/app/components/date-time-timer/date-time-timer.component';
import { DialogService } from 'src/app/lib-dialog/dialog.service';
import { StreamDto, StreamState } from 'src/app/lib-stream/stream-api.interface';

@Component({
  selector: 'app-panel-stream-admin',
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
  public streamDto: StreamDto | null = null;
  @Input()
  public countOfViewer: number | null = null;
  @Input()
  public broadcastDuration: string | null = null;

  @Output()
  readonly actionPrepare: EventEmitter<number> = new EventEmitter();
  @Output()
  readonly actionStart: EventEmitter<number> = new EventEmitter();
  @Output()
  readonly actionStop: EventEmitter<number> = new EventEmitter();
  @Output()
  readonly actionPause: EventEmitter<number> = new EventEmitter();

  public futureDate: Date;
  constructor(
    private translateService: TranslateService,
    private dialogService: DialogService,
  ) {
    this.futureDate = new Date('2024-10-31T22:10:00');
  }

  ngOnChanges(changes: SimpleChanges): void {
    // if (!!changes.streamDto && !!this.streamDTO?.credentials && this.streamDTO.state === StreamState.preparing) {
    //   this.doSettings(this.streamDTO.credentials);
    // }
  }

  ngOnInit(): void {
  }

  // ** Public API **

  public isBntPrepare(state: StreamState | undefined): boolean {
    const streamSate: StreamState = (state || StreamState.Waiting);
    return [StreamState.Waiting, StreamState.Stopped].includes(streamSate);
  }
  public isBntStart(state: StreamState | undefined): boolean {
    const streamSate: StreamState = (state || StreamState.Waiting);
    return [StreamState.Preparing, StreamState.Paused].includes(streamSate);
  }
  public isBntPause(state: StreamState | undefined): boolean {
    const streamSate: StreamState = (state || StreamState.Waiting);
    return [StreamState.Started].includes(streamSate);
  }
  public isBtnStop(state: StreamState | undefined): boolean {
    const streamSate: StreamState = (state || StreamState.Waiting);
    return [StreamState.Preparing, StreamState.Started, StreamState.Paused].includes(streamSate);
  }

  /*public doSettings(credentials: Credentials | null): void {
    if (!credentials || !credentials.rtmpPublishPath) { return; }
    const dataParams: SettingsData = {
      rtmpPath: credentials.rtmpPublishPath,
      rtmpStreamName: credentials.rtmpPublishStreamName
    };
    this.dialogService.openComponent(SettingsComponent, dataParams);
  }*/
  public doActionPrepare(): void {
    if (!!this.streamDto?.id) {
      this.actionPrepare.emit(this.streamDto.id);
    }
  }
  public doActionStart(): void {
    if (!!this.streamDto?.id) {
      this.actionStart.emit(this.streamDto.id);
    }
  }
  public doActionStop(): void {
    if (!!this.streamDto?.id) {
      const streamId = this.streamDto.id;
      const message = this.translateService.instant('panel-stream-admin.sure_you_want_stop_stream', { title: this.streamDto.title });
      const params = { btnNameCancel: 'buttons.no', btnNameAccept: 'buttons.yes' };
      this.dialogService.openConfirmation(message, '', params)
        .then((response) => {
          if (!!response) {
            this.actionStop.emit(streamId);
          }
        });
    }
  }
  public doActionPause(): void {
    if (!!this.streamDto?.id) {
      this.actionPause.emit(this.streamDto.id);
    }
  }

  // ** Private API **

}
