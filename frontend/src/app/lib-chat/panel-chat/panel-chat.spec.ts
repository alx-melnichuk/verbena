import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelChat } from './panel-chat';

describe('PanelChat', () => {
    let component: PanelChat;
    let fixture: ComponentFixture<PanelChat>;

    beforeEach(async () => {
        await TestBed.configureTestingModule({
            imports: [PanelChat]
        })
            .compileComponents();

        fixture = TestBed.createComponent(PanelChat);
        component = fixture.componentInstance;
        fixture.detectChanges();
    });

    it('should create', () => {
        expect(component).toBeTruthy();
    });
});
