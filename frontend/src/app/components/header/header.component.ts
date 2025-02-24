import { CommonModule } from '@angular/common';
import {
    ChangeDetectionStrategy, ChangeDetectorRef, Component, EventEmitter, HostBinding, HostListener, Input, OnChanges,
    Output, Renderer2, SimpleChanges, ViewEncapsulation
} from '@angular/core';
import { FormsModule } from '@angular/forms';
import { RouterLink, RouterLinkActive } from '@angular/router';
import { MatButtonModule } from '@angular/material/button';
import { MatMenuModule } from '@angular/material/menu';
import { MatSlideToggleModule } from '@angular/material/slide-toggle';
import { MatToolbarModule } from '@angular/material/toolbar';
import { TranslatePipe } from '@ngx-translate/core';

import { LOCALE_LIST, COLOR_SCHEME_LIST } from 'src/app/common/constants';
import { InitializationService } from 'src/app/common/initialization.service';
import { MainMenu, MainMenuUtil } from 'src/app/common/main-menu';
import { MAIN_MENU_LIST } from 'src/app/common/routes';
import { ProfileDto } from 'src/app/lib-profile/profile-api.interface';

export const HM_LOGOUT = 'logout';
export const HM_SET_LOCALE = 'setLocale';
export const HM_SET_COLOR_SCHEME = 'setColorScheme';

const CN_MIN_WINDOW_WIDTH = 768; // Minimum window width for displaying the main menu.
const CN_ResizeEventTimeout = 150; // milliseconds

@Component({
    selector: 'app-header',
    exportAs: 'appHeader',
    standalone: true,
    imports: [CommonModule, FormsModule, RouterLink, RouterLinkActive, MatButtonModule, MatMenuModule, MatSlideToggleModule,
        MatToolbarModule, TranslatePipe],
    templateUrl: './header.component.html',
    styleUrl: './header.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class HeaderComponent implements OnChanges {
    @Input()
    public currentRoute: string | null = null;
    @Input()
    public locale: string | null = null;
    @Input()
    public profileDto: ProfileDto | null = null;
    @Input()
    public colorScheme: string | null = null;

    @Output()
    readonly command: EventEmitter<Record<string, string>> = new EventEmitter();

    public nickname: string = '';
    public mainMenuItems: MainMenu[] = [];
    public panelMenuItems: MainMenu[] = [];
    public localeList: string[] = [...LOCALE_LIST];
    public colorSchemeList: string[] = [...COLOR_SCHEME_LIST];

    public isShowTheme: boolean = true;
    public isShowLocale: boolean = true;
    public isShowLogout: boolean = true;

    @HostBinding('class.h-is-authorized')
    get isAuthorizedVal(): boolean {
        return !!this.profileDto;
    }

    private timerResizeEvent: any = null;

    @HostListener('window:resize', ['$event'])
    public doScrollPanel(event: Event): void {
        event.preventDefault();
        event.stopPropagation();

        if (this.timerResizeEvent !== null) {
            clearTimeout(this.timerResizeEvent);
        }
        this.timerResizeEvent = setTimeout(() => {
            this.timerResizeEvent = null;

            this.prepareMenuItems(this.profileDto, MAIN_MENU_LIST, CN_MIN_WINDOW_WIDTH < this.getWidth());
            this.changeDetectorRef.markForCheck();
        }, CN_ResizeEventTimeout);
    }

    constructor(
        public renderer: Renderer2,
        public initializationService: InitializationService,
        private changeDetectorRef: ChangeDetectorRef,
    ) {
    }

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes['profileDto'] || !!changes['currentRoute']) {
            this.prepareMenuItems(this.profileDto, MAIN_MENU_LIST, CN_MIN_WINDOW_WIDTH < this.getWidth());
        }
    }

    // ** Public API **

    public doLogout(): void {
        this.doCommand(HM_LOGOUT, '');
    }

    public doSetLocale(value: string): void {
        this.doCommand(HM_SET_LOCALE, value);
    }

    public doSetColorScheme(value: string): void {
        this.doCommand(HM_SET_COLOR_SCHEME, value);
    }

    public nicknameWithSeparateSpaces(nickname: string | null | undefined): string {
        let result: string = nickname || '';
        if (!!result) {
            const ch1 = String.fromCharCode(0x200B); // "empty space" character for line breaks.
            if (result.indexOf('_') > -1) {
                result = result.replaceAll('_', '_' + ch1);
            } else {
                const idx = Math.round(result.length / 2);
                result = result.slice(0, idx) + ch1 + result.slice(idx);
            }
        }
        return result;
    }

    public prepareMenuItems(profileDto: ProfileDto | null, mainMenuList: string[], isShowMainMenu: boolean): void {
        this.nickname = this.nicknameWithSeparateSpaces(profileDto?.nickname);
        const items = MainMenuUtil.getList(profileDto != null, mainMenuList);
        this.mainMenuItems = isShowMainMenu ? items : [];
        this.panelMenuItems = isShowMainMenu ? [] : items;
    }

    // ** Private API **

    private getWidth(): number {
        return window.innerWidth || document.documentElement.clientWidth || document.body.clientWidth;
    }

    private doCommand(commandName: string, commandValue: string): void {
        this.command.emit({ [commandName]: commandValue });
    }
}
