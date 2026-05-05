import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelStreamCard } from './panel-stream-card';

describe('PanelStreamCard2', () => {
    let component: PanelStreamCard;
    let fixture: ComponentFixture<PanelStreamCard>;

    beforeEach(async () => {
        await TestBed.configureTestingModule({
            imports: [PanelStreamCard]
        })
            .compileComponents();

        fixture = TestBed.createComponent(PanelStreamCard);
        component = fixture.componentInstance;
        fixture.detectChanges();
    });

    it('should create', () => {
        expect(component).toBeTruthy();
    });
});
