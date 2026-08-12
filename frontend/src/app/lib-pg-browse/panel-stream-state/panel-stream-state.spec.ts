import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelStreamState } from './panel-stream-state';

describe('PanelStreamState', () => {
  let component: PanelStreamState;
  let fixture: ComponentFixture<PanelStreamState>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PanelStreamState]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PanelStreamState);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
