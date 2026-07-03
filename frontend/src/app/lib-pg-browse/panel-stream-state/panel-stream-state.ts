import { CommonModule } from "@angular/common";
import {
    ChangeDetectionStrategy, Component, HostBinding, Input, OnChanges, SimpleChanges, ViewEncapsulation
} from "@angular/core";
import { TranslatePipe } from "@ngx-translate/core";
import { StreamState } from "../../lib-stream/stream-dto";

@Component({
    selector: "app-panel-stream-state",
    exportAs: "appPanelStreamState",
    standalone: true,
    imports: [CommonModule, TranslatePipe],
    templateUrl: "./panel-stream-state.html",
    styleUrl: "./panel-stream-state.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelStreamState implements OnChanges {
    @Input()
    public streamState: StreamState | null | undefined = null;

    @HostBinding("attr.state")
    public get state() { return this.streamState?.toString(); }

    public strmStWaiting: StreamState = StreamState.waiting;
    public strmStPreparing: StreamState = StreamState.preparing;
    public strmStStarted: StreamState = StreamState.started;
    public strmStPaused: StreamState = StreamState.paused;
    public strmStStopped: StreamState = StreamState.stopped;
    public valueText: string | null = null;

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes["streamState"]) {
            this.streamState = this.streamState || StreamState.waiting;
            this.valueText = this.getValueText(this.streamState);
        }
    }

    // ** Private API **

    private getValueText(streamState: StreamState): string {
        let res = "";
        switch (streamState) {
            case StreamState.waiting: res = "panel-stream-state.has_not_started"; break;
            case StreamState.preparing: res = "panel-stream-state.has_not_started"; break;
            case StreamState.paused: res = "panel-stream-state.has_paused"; break;
            case StreamState.stopped: res = "panel-stream-state.has_ended"; break;
        }
        return res;
    }
}
