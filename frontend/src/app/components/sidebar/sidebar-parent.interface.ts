import { InjectionToken } from '@angular/core';

export interface SidebarParent {
    setIsOpen(isOpen: boolean): void;
    setIsOver(isOver: boolean): void;
}

export const APP_SIDEBAR_PARENT = new InjectionToken<SidebarParent>('APP_SIDEBAR_PARENT');
