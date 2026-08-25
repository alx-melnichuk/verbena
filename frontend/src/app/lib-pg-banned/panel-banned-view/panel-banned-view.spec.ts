import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelBannedView } from './panel-banned-view';

describe('PanelBannedView', () => {
  let component: PanelBannedView;
  let fixture: ComponentFixture<PanelBannedView>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PanelBannedView]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PanelBannedView);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
