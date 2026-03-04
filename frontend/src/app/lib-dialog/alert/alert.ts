import { CommonModule } from "@angular/common";
import {
    Component, ViewEncapsulation, ChangeDetectionStrategy, OnInit, Input, Output, EventEmitter, HostBinding, inject
} from "@angular/core";
import { TranslateService } from "@ngx-translate/core";
import { AlertMode } from "./alert-info";

@Component({
    selector: "app-alert",
    exportAs: "appAlert",
    standalone: true,
    imports: [CommonModule],
    templateUrl: "./alert.html",
    styleUrl: "./alert.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class Alert implements OnInit {
    @Input()
    public isOneLine: boolean | undefined = false;
    @Input()
    public closeable: boolean | undefined = true;
    @Input()
    public mode: AlertMode | undefined = AlertMode.comment;
    @Input()
    public title: string | undefined;
    @Input()
    public message: string | undefined;
    @Input()
    public messageHtml: string | undefined;

    @Output()
    readonly closed: EventEmitter<void> = new EventEmitter();

    @HostBinding("class.app-comment")
    public get isComment(): boolean {
        return this.mode === AlertMode.comment;
    }

    private translate: TranslateService = inject(TranslateService);

    @HostBinding("class.app-info")
    public get isInfo(): boolean {
        return this.mode === AlertMode.info;
    }

    @HostBinding("class.app-warning")
    public get isWarning(): boolean {
        return this.mode === AlertMode.warning;
    }

    @HostBinding("class.app-error")
    public get isError(): boolean {
        return this.mode === AlertMode.error;
    }

    @HostBinding("class.app-success")
    public get isSuccess(): boolean {
        return this.mode === AlertMode.success;
    }

    ngOnInit(): void {
        this.title = !!this.title ? this.translate.instant(this.title) : this.title;
        this.message = !!this.message ? this.translate.instant(this.message) : this.message;
    }

    // ** Public API **

    public doClose(): void {
        this.closed.emit();
    }
}
