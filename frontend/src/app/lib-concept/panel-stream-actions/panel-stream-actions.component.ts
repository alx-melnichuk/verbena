import { ChangeDetectionStrategy, Component, EventEmitter, Input, Output, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatButtonModule } from '@angular/material/button';
import { TranslatePipe, TranslateService } from '@ngx-translate/core';

import { DialogService } from 'src/app/lib-dialog/dialog.service';
import { StreamState } from 'src/app/lib-stream/stream-api.interface';

@Component({
    selector: 'app-panel-stream-actions',
    exportAs: 'appPanelStreamActions',
    standalone: true,
    imports: [CommonModule, MatButtonModule, TranslatePipe],
    templateUrl: './panel-stream-actions.component.html',
    styleUrl: './panel-stream-actions.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelStreamActionsComponent {
    @Input()
    public state: StreamState | null | undefined;
    @Input()
    public title: string | null | undefined;

    @Output()
    readonly changeState: EventEmitter<StreamState> = new EventEmitter();

    public statePreparing = StreamState.preparing;
    public stateStarted = StreamState.started;
    public statePaused = StreamState.paused;
    public stateStopped = StreamState.stopped;

    constructor(
        private translateService: TranslateService,
        private dialogService: DialogService,
    ) {
    }

    // ** Public API **

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
    public isShowBtnStop(state: StreamState | null | undefined): boolean {
        const state2: StreamState = (state || StreamState.waiting);
        return [StreamState.preparing, StreamState.started, StreamState.paused].includes(state2);
    }

    public doChangeState(newState: StreamState): void {
        if (this.state != null) {
            if (newState == StreamState.stopped) {
                const message = this.translateService.instant('panel-stream-actions.sure_you_want_stop_stream', { title: this.title });
                const params = { btnNameCancel: 'buttons.no', btnNameAccept: 'buttons.yes' };
                this.dialogService.openConfirmation(message, '', params)
                    .then((response) => {
                        if (!!response) {
                            this.changeState.emit(StreamState.stopped);
                        }
                    });
            } else {
                this.changeState.emit(newState);
            }
        }
    }

    // ** Private API **

}
