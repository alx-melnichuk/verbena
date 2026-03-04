import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelStreamActions } from './panel-stream-actions';

describe('PanelStreamActions', () => {
  let component: PanelStreamActions;
  let fixture: ComponentFixture<PanelStreamActions>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PanelStreamActions]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PanelStreamActions);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
