import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelProfileRemove } from './panel-profile-remove';

describe('PanelProfileRemove', () => {
  let component: PanelProfileRemove;
  let fixture: ComponentFixture<PanelProfileRemove>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PanelProfileRemove]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PanelProfileRemove);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
