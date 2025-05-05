% PLOTTING
win_scale = 15;
window = [-win_scale, win_scale, -win_scale, win_scale];
movieFlag = false; % Set to true if you want to save a video
hFig = figure(1); clf;


t = 25; % simulation time, [sec]
dt = 0.1; % time step, [sec]
K = t/dt; % number of frames = time of animation / time for each frame

if movieFlag
    v = VideoWriter('movie', 'MPEG-4'); 
    v.FrameRate = 1/dt;
    open(v); 
end

% ______________Initial parameters________________

N = 20; % number of individuals
alpha = 270; % vision volume, [deg]
rr = 1; % zone of repulsion [units]
ro = 5; % zone of orientation [units]
ra = 15; % zone of attraction [units]
omega = 40; % max rotationa rate, [deg/sec]
s = 3; % speed of each individual, [units/sec]

% INITIALIZE INDIVIDUALS

% ____random____
r = 5*rand(3,N) - 2.5; % random positions around (0,0)
v = rand(3,N) - 0.5; % random velocity directions
%r(3,:) = 0;
%v(3,:) = 0;

% ____example 1____
%r(:,1) = [5; 0; 0]; r(:,2) = [-5; 0; 0];
%v(:,1) = [-1; 0; 0]; v(:,2) = [1; -1; 0];

%____example 2____
%r(:,1) = [5; 0; 0]; r(:,2) = [-5; 0; 0]; r(:,3) = [0; 0; 0]
%v(:,1) = [0; 1; 0]; v(:,2) = [0; 1; 0]; v(:,3) = [0; -1; 0]

for i = 1:N % normalize these vectors
    v(:,i) = v(:,i)/norm(v(:,i));
end

% matrices for information about the system, recorded each frame
r_group = zeros(3,K); % COM Position
v_group = zeros(3,K); % COM velocity
p_group = zeros(3,K); % COM linear momentum (equal to velocity if mass of system = 1)
h_group = zeros(3,K); % angular momentum about COM
r_inter = zeros(3,K); % distance of each fish from COM, used in calculating h_group

% ______________________Loop for each frame to be rendered___________________
for k = 1:K
    dir = zeros(3,N); % desired direction for each fish at the end of the frame
    
    for n = 1:N
        % variable reset
        dis = zeros(3,N); % distance vectors between this fish and all others
        dirTemp = zeros(3,1); % temporary desired direction
        %dir = zeros(3,N); 
        tempIndex = zeros(1,N); % classify other fish: 0 = repulsion, 1 = orientation, 2 = attraction, 3 = ignore
        temp2 = zeros(3,N); % temporary matrix for location, velocity, etc. of important fish
        temp3 = zeros(3,N); % ^^
        inrr = false; % flags if there are fish in repulsion zone
        inro = false; % ^^ orientation zone
        inra = false; % ^^ attraction zone
        angInit = 0; % initial angle calculated from velocity vector
        angTarg = 0; % target angle calcualted from dir vector
        dif1 = 0; dif2 = 0; % temporary variables for angular distance calculations
        angFinal = 0; % final angle to turn to
        
        % __________________Determine zone for all other fish_________________
        for i = 1:N
            dis(:,i) = r(:,i) - r(:,n); % vector from fish n to fish i
            d = norm(dis(:,i)); % distance between fish n and fish i
            
            %_____blind spot______
            % calculate angle between where fish n is facing and fish i's position
            angle = acosd(dot(v(:,n), dis(:,i)) / norm(v(:,n)) / norm(dis(:,i)));    
           
            % if fish i is outside fish n's field of view (blind zone)
            if angle > 0.5 * alpha
                tempIndex(1,i) = 3; % ignore (3)
            end
            
            % classify distance into repulsion/orientation/attraction
            if d <= rr & tempIndex(1,i) ~= 3
                tempIndex(1,i) = 0; % repulsion zone (0)
            elseif d <= ro & tempIndex(1,i) ~= 3
                tempIndex(1,i) = 1; % orientation zone (1)
            elseif d <= ra & tempIndex(1,i) ~= 3
                tempIndex(1,i) = 2; % attraction zone (2)
            else
                tempIndex(1,i) = 3; % out of range, ignore (3)
            end
    
            tempIndex(1,n) = 3; % ignore self (3)                   
        end

        % _____________repulsion zone______________
        
        fishCount = 0;
        for i = 1:N
            if tempIndex(1,i) == 0
                inrr = true; % true if >=1 fish in repulsion zone
                fishCount = fishCount + 1;
            end
        end

        if inrr
            for i = 1:N
                if tempIndex(1,i) == 0
                    temp2(:,i) = r(:,i); % saves position of fish in repulsion zone
                end
            end

            % mean position of repulsion zone fish
            target = [sum(temp2(1,:) / fishCount); sum(temp2(2,:) / fishCount); 0]; 
            % head directly away from average
            dir(:,n) = -(r(:,n) - target) / norm(r(:,n) - target);     
            continue; % cancels loop for particular fish, as all other fish outside repulsion zone
        end
        
    
        % _____________orientation zone______________

        fishCount = 0;
        for i = 1:N
            if tempIndex(1,i) == 1
                inro = true; % true if >=1 fish in orientation zone
                fishCount = fishCount + 1;
            end
        end
    
        if inro
            for i = 1:N
                if tempIndex(1,i) == 1
                    temp2(:,i) = v(:,i); % saves velocity of fish in orientation zone
                end
            end

            % mean velocity of orientation zone fish
            target = [sum(temp2(1,:) / fishCount); sum(temp2(2,:) / fishCount); 0];
            dirTemp = target / norm(target); % average direction
        end
    
        % _____________attraction zone______________

        fishCount = 0;
        for i = 1:N
            if tempIndex(1, i) == 2
                inra = true; % true if >=1 fish in attraction zone
                fishCount = fishCount + 1;
            end
        end
    
        if inra
            for i = 1:N
                if tempIndex(1,i) == 2
                    temp3(:,i) = r(:,i); % saves position of fish in attraction zone
                end
            end
            % mean position of attraction zone fish
            target = [sum(temp3(1,:) / fishCount); sum(temp3(2,:) / fishCount); 0];
            dir(:,n) = -(r(:,n) - target) / norm(r(:,n) - target); % move towards center
        end
        
        % _____________combine zones______________

        if inro
            if inra
                % mostly attraction, some orientation
                dir(:,n) = (0.8 * dir(:,n) + 0.2 * dirTemp(:,1));
                dir(:,n) = dir(:,n) / norm(dir(:,n)); % normalize
            else
                dir(:,n) = dirTemp(:,1); % only orientation
            end
        end
    
    end

    %__________________________________MOVEMENT______________________________
    
    for i = 1:N 
        
        % no fish in any zone, move in current direction
        if dir(:,i) == [0;0;0]
            r(:,i) = r(:,i) + s * dt * v(:,i);
            continue;
        end

        angFinal = 0;
        
        % current velocity angle (deg)
        angInit = atan2d(v(2,i), v(1,i)); 
        if angInit < 0
            angInit = angInit + 360;
        end

        % target angle (deg)
        angTarg = atan2d(dir(2,i), dir(1,i));
        if angTarg < 0
            angTarg = angTarg + 360;
        end

        % calculates clockwise and counterclockwise distances between angles
        dif1 = angTarg - angInit;
        dif2 = 360 - abs(dif1);
        if dif1 > 0 
            % first distance is pos, set dif1 -> pos, dif2 -> neg
            counterclockwise = dif1;
            clockwise = dif2;
        else
            % first distance is neg, set abs(dif1) -> neg, dif2 -> pos
            counterclockwise = dif2;
            clockwise = abs(dif1);
        end
        
        % checks which direction is shortest, applies movement in that direction
        % omega * dt for smooth turning relative to time
        if counterclockwise < clockwise 
            if counterclockwise < omega * dt % makes sure fish will not overshoot turn
                angFinal = angTarg;
            else
                angFinal = angInit + omega * dt;
            end
        else
            if clockwise < omega * dt
                angFinal = angTarg;
            else
                angFinal = angInit - omega * dt;
            end
        end
        
        % clamps angle to [0, 360] range
        if angFinal > 360
            angFinal = angFinal - 360;
        elseif angFinal < 0
            angFinal = 360 + angFinal;
        end
        % set new velocity direction, and update position (move)
        v(:,i) = [cosd(angFinal); sind(angFinal); 0];
        r(:,i) = r(:,i) +  s * dt * v(:,i);
    end

    % _____________________________MOMENTUM_ETC_________________________

    % center of mass / mean position of all fish
    r_group(:,k) = [mean(r(1,:)); mean(r(2,:)); 0];
    
    % average velocity / mean velocity of all fish
    v_group(:,k) = s * [mean(v(1,:)); mean(v(2,:)); 0];

    % linear momentum / mean velocity of all fish
    p_group(:,k) = s/N * [sum(v(1,:)); sum(v(2,:)); 0];

    % angular momentum
    % for each fish, calculate distance from COM, then cross product with velocity
    for j = 1:N
        r_inter(1,j) = r(1,j) - r_group(1,k);
        r_inter(2,j) = r(2,j) - r_group(2,k);
        
        h_group(:,k) = h_group(:,k) + s/N * cross(r_inter(:,j), v(:,j));
    end

    %_______Animation stuff__________
    figure(hFig);
    plot(r(1,:), r(2,:), 'o'); hold on; % fish positions/body
    quiver(r(1,:), r(2,:), v(1,:), v(2,:), 0); hold off; % fish velocity vectors/arrows
    titleStr = sprintf('t = %2.2f', k*dt);
    title(titleStr);
    axis equal;
    avgWindow = [sum(r(1,:))/N*[1,1], sum(r(2,:))/N*[1,1]];
    axis(window + avgWindow);
    drawnow;
    pause(0.25);

    if movieFlag
        frame = getframe(hFig);
        writeVideo(v, frame);
    end
   
end

if movieFlag
    close(v); % Close video file
end

% ___________________PLOTTING_____________________
tspan = dt : dt : t;

% plot center of mass over time
plot(tspan, r_group(1,:), tspan, r_group(2,:));
xlim([tspan(1) tspan(K)]); % sets the x axis limits to tspan 
xlabel('Time (sec)') ;
ylabel('Center of mass');
legend('x','y');
clf

% plot average velocity/linear momentum 
plot(tspan, v_group(1,:), tspan, v_group(2,:));
xlim([tspan(1) tspan(K)]); % sets the x axis limits to tspan 
xlabel('Time (sec)') ;
ylabel('Velocity/linear momentum of C.O.M.');
legend('x','y');
clf

% plot angular momentum (z-axis only) 
plot(tspan, h_group(3,:), tspan, h_group(3,:));
xlim([tspan(1) tspan(K)]); % sets the x axis limits to tspan 
xlabel('Time (sec)') ;
ylabel('Angular Momentum around C.O.M.');
clf