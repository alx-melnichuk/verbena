import { CommonModule } from "@angular/common";
import { ChangeDetectionStrategy, Component, HostBinding, Input, ViewEncapsulation } from "@angular/core";
import { RouterLink } from "@angular/router";
import { TranslatePipe } from "@ngx-translate/core";
import { ROUTE_ABOUT } from "../../common/routes";

@Component({
    selector: "app-footer",
    exportAs: "appFooter",
    standalone: true,
    imports: [CommonModule, RouterLink, TranslatePipe],
    templateUrl: "./footer.html",
    styleUrl: "./footer.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class Footer {
    @Input()
    public isAuthorized: boolean | null | undefined;

    @HostBinding("class.f-is-authorized")
    get isAuthorizedVal(): boolean {
        return !!this.isAuthorized;
    }

    @HostBinding("class.app-pn-bg")
    get isAppPnBg(): boolean {
        return true;
    }

    linkAbout = ROUTE_ABOUT;

    // ** Public API **

    // ** Private API **

}
